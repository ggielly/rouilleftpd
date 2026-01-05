// Gestion des statistiques de transfert - inspiré de glFTPd

use crate::core_quota::error::QuotaError;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

/// Statistiques de transfert pour un utilisateur
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserTransferStats {
    /// Nom d'utilisateur
    pub username: String,

    /// Octets uploadés (cumulatif)
    pub total_uploaded: u64,

    /// Octets téléchargés (cumulatif)
    pub total_downloaded: u64,

    /// Nombre de fichiers uploadés
    pub files_uploaded: u32,

    /// Nombre de fichiers téléchargés
    pub files_downloaded: u32,

    /// Dernière activité (en timestamp)
    pub last_activity: Option<u64>,
}

impl UserTransferStats {
    pub fn new(username: &str) -> Self {
        Self {
            username: username.to_string(),
            total_uploaded: 0,
            total_downloaded: 0,
            files_uploaded: 0,
            files_downloaded: 0,
            last_activity: Some(chrono::Local::now().timestamp() as u64),
        }
    }

    /// Met à jour les statistiques après un upload
    pub fn record_upload(&mut self, bytes: u64) {
        self.total_uploaded += bytes;
        self.files_uploaded += 1;
        self.last_activity = Some(chrono::Local::now().timestamp() as u64);
    }

    /// Met à jour les statistiques après un download
    pub fn record_download(&mut self, bytes: u64) {
        self.total_downloaded += bytes;
        self.files_downloaded += 1;
        self.last_activity = Some(chrono::Local::now().timestamp() as u64);
    }

    /// Formate les statistiques pour l'affichage
    pub fn format_stats(&self) -> String {
        let upload_mb = self.total_uploaded as f64 / (1024.0 * 1024.0);
        let download_mb = self.total_downloaded as f64 / (1024.0 * 1024.0);

        format!(
            "Upload: {:.2}MB ({} files), Download: {:.2}MB ({} files)",
            upload_mb, self.files_uploaded, download_mb, self.files_downloaded
        )
    }
}

/// Gestionnaire de statistiques pour tous les utilisateurs
#[derive(Debug, Clone)]
pub struct TransferStatsManager {
    /// Statistiques par utilisateur
    stats: HashMap<String, UserTransferStats>,

    /// Fichier de stockage
    storage_file: PathBuf,
}

impl TransferStatsManager {
    pub fn new(storage_file: PathBuf) -> Self {
        Self {
            stats: HashMap::new(),
            storage_file,
        }
    }

    /// Charge les statistiques depuis le fichier
    pub fn load(&mut self) -> Result<(), QuotaError> {
        if !self.storage_file.exists() {
            return Ok(());
        }

        let content = std::fs::read_to_string(&self.storage_file)
            .map_err(|e| QuotaError::QuotaReadError(e.to_string()))?;

        let loaded: HashMap<String, UserTransferStats> = serde_json::from_str(&content)
            .map_err(|e| QuotaError::QuotaReadError(e.to_string()))?;

        self.stats = loaded;
        Ok(())
    }

    /// Sauvegarde les statistiques dans le fichier
    pub fn save(&self) -> Result<(), QuotaError> {
        let content = serde_json::to_string(&self.stats)
            .map_err(|e| QuotaError::QuotaWriteError(e.to_string()))?;

        std::fs::write(&self.storage_file, content)
            .map_err(|e| QuotaError::QuotaWriteError(e.to_string()))?;

        Ok(())
    }

    /// Obtient ou crée les statistiques pour un utilisateur
    pub fn get_or_create_stats(&mut self, username: &str) -> &mut UserTransferStats {
        self.stats
            .entry(username.to_string())
            .or_insert_with(|| UserTransferStats::new(username))
    }

    /// Met à jour les statistiques après un upload
    pub fn record_upload(&mut self, username: &str, bytes: u64) {
        let stats = self.get_or_create_stats(username);
        stats.record_upload(bytes);
    }

    /// Met à jour les statistiques après un download
    pub fn record_download(&mut self, username: &str, bytes: u64) {
        let stats = self.get_or_create_stats(username);
        stats.record_download(bytes);
    }

    /// Obtient les statistiques pour un utilisateur
    pub fn get_stats(&self, username: &str) -> Option<&UserTransferStats> {
        self.stats.get(username)
    }
}
