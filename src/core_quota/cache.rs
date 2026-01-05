use crate::core_quota::disk_io::DiskIoPool;
use crate::core_quota::error::QuotaError;
use crate::core_quota::quota::UserQuota;
use crate::core_quota::ratio::UserRatio;
use crate::core_quota::stats::TransferStatsManager;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Structure pour gérer le cache des quotas
#[derive(Debug)]
pub struct QuotaCache {
    /// Cache des quotas utilisateurs
    user_quotas: Arc<RwLock<HashMap<String, UserQuota>>>,

    /// Cache des ratios utilisateurs
    user_ratios: Arc<RwLock<HashMap<String, UserRatio>>>,

    /// Gestionnaire de statistiques
    pub stats_manager: Arc<RwLock<TransferStatsManager>>,

    /// Chemin du fichier de stockage des quotas
    quota_storage_file: PathBuf,

    /// Chemin du fichier de stockage des ratios
    ratio_storage_file: PathBuf,

    /// Chemin du fichier de stockage des statistiques
    stats_storage_file: PathBuf,

    /// Pool d'E/S disque pour optimiser les accès
    disk_io_pool: Arc<DiskIoPool>,

    /// Indicateur de modification pour les quotas
    quotas_dirty: Arc<RwLock<bool>>,

    /// Indicateur de modification pour les ratios
    ratios_dirty: Arc<RwLock<bool>>,

    /// Indicateur de modification pour les statistiques
    stats_dirty: Arc<RwLock<bool>>,
}

impl QuotaCache {
    pub fn new(
        quota_storage_file: PathBuf,
        ratio_storage_file: PathBuf,
        stats_storage_file: PathBuf,
    ) -> Self {
        Self {
            user_quotas: Arc::new(RwLock::new(HashMap::new())),
            user_ratios: Arc::new(RwLock::new(HashMap::new())),
            stats_manager: Arc::new(RwLock::new(TransferStatsManager::new(
                stats_storage_file.clone(),
            ))),
            quota_storage_file,
            ratio_storage_file,
            stats_storage_file,
            disk_io_pool: Arc::new(DiskIoPool::new()),
            quotas_dirty: Arc::new(RwLock::new(false)),
            ratios_dirty: Arc::new(RwLock::new(false)),
            stats_dirty: Arc::new(RwLock::new(false)),
        }
    }

    /// Charge tous les données depuis les fichiers
    pub async fn load_all(&self) -> Result<(), QuotaError> {
        // Charger les quotas
        self.load_quotas().await?;

        // Charger les ratios
        self.load_ratios().await?;

        // Charger les statistiques
        self.load_stats().await?;

        // Réinitialiser les indicateurs de modification
        *self.quotas_dirty.write().await = false;
        *self.ratios_dirty.write().await = false;
        *self.stats_dirty.write().await = false;

        Ok(())
    }

    /// Charge les quotas depuis le fichier
    async fn load_quotas(&self) -> Result<(), QuotaError> {
        if self.quota_storage_file.exists() {
            let content = self
                .disk_io_pool
                .read_file(&self.quota_storage_file)
                .await
                .map_err(|e| QuotaError::QuotaReadError(e.to_string()))?;

            let quotas: HashMap<String, UserQuota> = serde_json::from_str(&content)
                .map_err(|e| QuotaError::QuotaReadError(e.to_string()))?;

            *self.user_quotas.write().await = quotas;
        }
        Ok(())
    }

    /// Charge les ratios depuis le fichier
    async fn load_ratios(&self) -> Result<(), QuotaError> {
        if self.ratio_storage_file.exists() {
            let content = self
                .disk_io_pool
                .read_file(&self.ratio_storage_file)
                .await
                .map_err(|e| QuotaError::QuotaReadError(e.to_string()))?;

            let ratios: HashMap<String, UserRatio> = serde_json::from_str(&content)
                .map_err(|e| QuotaError::QuotaReadError(e.to_string()))?;

            *self.user_ratios.write().await = ratios;
        }
        Ok(())
    }

    /// Charge les statistiques depuis le fichier
    async fn load_stats(&self) -> Result<(), QuotaError> {
        self.stats_manager.write().await.load()?;
        Ok(())
    }

    /// Sauvegarde tous les données dans les fichiers
    pub async fn save_all(&self) -> Result<(), QuotaError> {
        // Sauvegarder les quotas si modifiés
        if *self.quotas_dirty.read().await {
            self.save_quotas().await?;
            *self.quotas_dirty.write().await = false;
        }

        // Sauvegarder les ratios si modifiés
        if *self.ratios_dirty.read().await {
            self.save_ratios().await?;
            *self.ratios_dirty.write().await = false;
        }

        // Sauvegarder les statistiques si modifiées
        if *self.stats_dirty.read().await {
            self.save_stats().await?;
            *self.stats_dirty.write().await = false;
        }

        Ok(())
    }

    /// Sauvegarde les quotas dans le fichier
    async fn save_quotas(&self) -> Result<(), QuotaError> {
        let quotas = self.user_quotas.read().await;
        let content = serde_json::to_string(&*quotas)
            .map_err(|e| QuotaError::QuotaWriteError(e.to_string()))?;

        self.disk_io_pool
            .write_file(self.quota_storage_file.clone(), content)
            .await
            .map_err(|e| QuotaError::QuotaWriteError(e.to_string()))?;

        Ok(())
    }

    /// Sauvegarde les ratios dans le fichier
    async fn save_ratios(&self) -> Result<(), QuotaError> {
        let ratios = self.user_ratios.read().await;
        let content = serde_json::to_string(&*ratios)
            .map_err(|e| QuotaError::QuotaWriteError(e.to_string()))?;

        self.disk_io_pool
            .write_file(self.ratio_storage_file.clone(), content)
            .await
            .map_err(|e| QuotaError::QuotaWriteError(e.to_string()))?;

        Ok(())
    }

    /// Sauvegarde les statistiques dans le fichier
    async fn save_stats(&self) -> Result<(), QuotaError> {
        self.stats_manager.read().await.save()?;
        Ok(())
    }

    /// Obtient ou crée un quota pour un utilisateur
    pub async fn get_or_create_user_quota(
        &self,
        username: &str,
        base_dir: PathBuf,
    ) -> Result<UserQuota, QuotaError> {
        {
            let quotas = self.user_quotas.read().await;
            if let Some(quota) = quotas.get(username) {
                return Ok(quota.clone());
            }
        }

        // Si le quota n'existe pas, créer un nouveau quota
        let quota = UserQuota::new(username, 10737418240, base_dir); // 10GB par défaut

        {
            let mut quotas = self.user_quotas.write().await;
            quotas.insert(username.to_string(), quota.clone());
        }

        // Marquer comme modifié
        *self.quotas_dirty.write().await = true;

        Ok(quota)
    }

    /// Obtient ou crée un ratio pour un utilisateur
    pub async fn get_or_create_user_ratio(&self, username: &str) -> Result<UserRatio, QuotaError> {
        {
            let ratios = self.user_ratios.read().await;
            if let Some(ratio) = ratios.get(username) {
                return Ok(ratio.clone());
            }
        }

        // Si le ratio n'existe pas, créer un nouveau ratio
        let ratio = UserRatio::new(username, "1:1")?;

        {
            let mut ratios = self.user_ratios.write().await;
            ratios.insert(username.to_string(), ratio.clone());
        }

        // Marquer comme modifié
        *self.ratios_dirty.write().await = true;

        Ok(ratio)
    }

    /// Met à jour un quota utilisateur
    pub async fn update_user_quota(
        &self,
        username: &str,
        quota: UserQuota,
    ) -> Result<(), QuotaError> {
        {
            let mut quotas = self.user_quotas.write().await;
            quotas.insert(username.to_string(), quota);
        }

        // Marquer comme modifié
        *self.quotas_dirty.write().await = true;

        Ok(())
    }

    /// Met à jour un ratio utilisateur
    pub async fn update_user_ratio(
        &self,
        username: &str,
        ratio: UserRatio,
    ) -> Result<(), QuotaError> {
        {
            let mut ratios = self.user_ratios.write().await;
            ratios.insert(username.to_string(), ratio);
        }

        // Marquer comme modifié
        *self.ratios_dirty.write().await = true;

        Ok(())
    }

    /// Met à jour les statistiques d'un utilisateur
    pub async fn update_user_stats(
        &self,
        username: &str,
        bytes: u64,
        is_upload: bool,
    ) -> Result<(), QuotaError> {
        {
            let mut stats_manager = self.stats_manager.write().await;
            if is_upload {
                stats_manager.record_upload(username, bytes);
            } else {
                stats_manager.record_download(username, bytes);
            }
        }

        // Marquer comme modifié
        *self.stats_dirty.write().await = true;

        Ok(())
    }

    /// Obtient les quotas
    pub async fn get_quotas(&self) -> HashMap<String, UserQuota> {
        self.user_quotas.read().await.clone()
    }

    /// Obtient les ratios
    pub async fn get_ratios(&self) -> HashMap<String, UserRatio> {
        self.user_ratios.read().await.clone()
    }

    /// Force la sauvegarde de toutes les données
    pub async fn force_save_all(&self) -> Result<(), QuotaError> {
        self.disk_io_pool
            .flush()
            .await
            .map_err(|e| QuotaError::QuotaWriteError(e.to_string()))?;
        Ok(())
    }
}
