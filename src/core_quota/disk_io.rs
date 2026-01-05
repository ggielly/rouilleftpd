use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::sync::{Notify, RwLock};
use tokio::time::{sleep, Duration};

/// Structure pour gérer les écritures batchées
pub struct BatchWriter {
    /// Données en attente d'écriture
    pending_writes: Arc<RwLock<HashMap<PathBuf, String>>>,

    /// Notification pour le batch
    notify: Arc<Notify>,

    /// Indicateur d'arrêt
    shutdown: Arc<RwLock<bool>>,
}

impl BatchWriter {
    pub fn new() -> Self {
        let batch_writer = Self {
            pending_writes: Arc::new(RwLock::new(HashMap::new())),
            notify: Arc::new(Notify::new()),
            shutdown: Arc::new(RwLock::new(false)),
        };

        // Démarrer le service de batch en arrière-plan
        batch_writer.start_batch_service();

        batch_writer
    }

    /// Démarrer le service de batch en arrière-plan
    fn start_batch_service(&self) {
        let pending_writes = Arc::clone(&self.pending_writes);
        let notify = Arc::clone(&self.notify);
        let shutdown = Arc::clone(&self.shutdown);

        tokio::spawn(async move {
            loop {
                // Attendre une notification ou un délai de 5 secondes
                tokio::select! {
                    _ = notify.notified() => {
                        // Réinitialiser la notification
                    }
                    _ = sleep(Duration::from_secs(5)) => {
                        // Délai d'attente atteint, écrire les données
                    }
                }

                // Vérifier si on doit s'arrêter
                if *shutdown.read().await {
                    break;
                }

                // Écrire toutes les données en attente
                Self::flush_pending_writes(&pending_writes).await;
            }

            // Lors de l'arrêt, écrire toutes les données restantes
            Self::flush_pending_writes(&pending_writes).await;
        });
    }

    /// Écrire des données dans le batch
    pub async fn write(&self, path: PathBuf, content: String) -> Result<(), std::io::Error> {
        {
            let mut pending = self.pending_writes.write().await;
            pending.insert(path, content);
        }

        // Notifier le service de batch
        self.notify.notify_one();

        Ok(())
    }

    /// Forcer l'écriture immédiate de toutes les données en attente
    pub async fn flush(&self) -> Result<(), std::io::Error> {
        Self::flush_pending_writes(&self.pending_writes).await;
        Ok(())
    }

    /// Écrire toutes les données en attente
    async fn flush_pending_writes(pending_writes: &Arc<RwLock<HashMap<PathBuf, String>>>) {
        let mut pending = pending_writes.write().await;

        if pending.is_empty() {
            return;
        }

        // Créer une copie des données à écrire
        let to_write = pending.clone();
        pending.clear();

        // Libérer le verrou avant les écritures disque
        drop(pending);

        // Écrire toutes les données
        for (path, content) in to_write {
            if let Err(e) = tokio::fs::write(&path, content).await {
                eprintln!(
                    "Erreur lors de l'écriture du fichier {}: {}",
                    path.display(),
                    e
                );
            }
        }
    }

    /// Arrêter le service de batch
    pub async fn shutdown(&self) {
        *self.shutdown.write().await = true;
        self.notify.notify_one();
    }
}

impl Drop for BatchWriter {
    fn drop(&mut self) {
        // On ne peut pas faire d'async dans drop, donc on se contente de l'indiquer
        // L'arrêt sera géré par un appel explicite à shutdown()
    }
}

// Service de cache pour les lectures fréquentes
use std::time::Duration as StdDuration;

pub struct ReadCache {
    /// Cache des lectures de fichiers
    cache: Arc<RwLock<HashMap<PathBuf, (String, u64)>>>, // (content, timestamp in seconds)

    /// Durée de vie des entrées dans le cache (5 minutes par défaut)
    ttl: StdDuration,
}

impl ReadCache {
    pub fn new() -> Self {
        Self {
            cache: Arc::new(RwLock::new(HashMap::new())),
            ttl: StdDuration::from_secs(300), // 5 minutes
        }
    }

    /// Lire un fichier avec mise en cache
    pub async fn read_file(&self, path: &PathBuf) -> Result<String, std::io::Error> {
        // Vérifier si le fichier est dans le cache
        {
            let cache = self.cache.read().await;
            if let Some((content, timestamp)) = cache.get(path) {
                let current_time = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_secs();

                if current_time - timestamp < self.ttl.as_secs() {
                    return Ok(content.clone());
                }
            }
        }

        // Lire le fichier depuis le disque
        let content = tokio::fs::read_to_string(path).await?;

        // Mettre à jour le cache
        {
            let mut cache = self.cache.write().await;
            let current_time = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs();
            cache.insert(path.clone(), (content.clone(), current_time));
        }

        Ok(content)
    }

    /// Invalider une entrée du cache
    pub async fn invalidate(&self, path: &PathBuf) {
        let mut cache = self.cache.write().await;
        cache.remove(path);
    }

    /// Nettoyer les entrées expirées
    pub async fn cleanup(&self) {
        let current_time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        let ttl_secs = self.ttl.as_secs();

        let mut cache = self.cache.write().await;
        cache.retain(|_, (_, timestamp)| current_time - *timestamp < ttl_secs);
    }
}

// Service de pooling pour les opérations de lecture/écriture
#[derive(Clone)]
pub struct DiskIoPool {
    /// Batch writer pour les écritures
    batch_writer: Arc<BatchWriter>,

    /// Cache pour les lectures
    read_cache: Arc<ReadCache>,
}

impl std::fmt::Debug for DiskIoPool {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DiskIoPool")
            .field("batch_writer", &"BatchWriter { ... }")
            .field("read_cache", &"ReadCache { ... }")
            .finish()
    }
}

impl DiskIoPool {
    pub fn new() -> Self {
        Self {
            batch_writer: Arc::new(BatchWriter::new()),
            read_cache: Arc::new(ReadCache::new()),
        }
    }

    /// Lire un fichier avec mise en cache
    pub async fn read_file(&self, path: &PathBuf) -> Result<String, std::io::Error> {
        self.read_cache.read_file(path).await
    }

    /// Écrire un fichier avec batch
    pub async fn write_file(&self, path: PathBuf, content: String) -> Result<(), std::io::Error> {
        self.batch_writer.write(path, content).await
    }

    /// Forcer l'écriture immédiate de toutes les données en attente
    pub async fn flush(&self) -> Result<(), std::io::Error> {
        self.batch_writer.flush().await
    }

    /// Arrêter le pool d'E/S disque
    pub async fn shutdown(&self) {
        self.batch_writer.shutdown().await;
    }
}

impl Default for DiskIoPool {
    fn default() -> Self {
        Self::new()
    }
}
