// Gestionnaire principal des quotas et ratios
// Ce module centralise toute la logique de gestion des quotas/ratios

use crate::core_quota::cache::QuotaCache;
use crate::core_quota::{
    config::{GroupQuotaConfig, QuotaConfig, UserQuotaConfig},
    error::QuotaError,
    quota::UserQuota,
    ratio::UserRatio,
};
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::{Mutex, RwLock};

/// Gestionnaire principal des quotas et ratios
#[derive(Clone)]
pub struct QuotaManager {
    /// Configuration globale
    config: Arc<QuotaConfig>,

    /// Configuration des groupes
    group_config: Arc<Mutex<GroupQuotaConfig>>,

    /// Configuration des utilisateurs
    user_config: Arc<Mutex<UserQuotaConfig>>,

    /// Cache des quotas et ratios
    cache: Arc<QuotaCache>,
}

impl QuotaManager {
    /// Crée un nouveau gestionnaire de quotas
    pub fn new(
        config: QuotaConfig,
        group_config: GroupQuotaConfig,
        user_config: UserQuotaConfig,
    ) -> Self {
        let cache = Arc::new(QuotaCache::new(
            config.quota_storage_file.clone(),
            config.ratio_storage_file.clone(),
            config.stats_storage_file.clone(),
        ));

        Self {
            config: Arc::new(config),
            group_config: Arc::new(Mutex::new(group_config)),
            user_config: Arc::new(Mutex::new(user_config)),
            cache,
        }
    }

    /// Charge les données de quota depuis les fichiers
    pub async fn load(&self) -> Result<(), QuotaError> {
        // Charger les données dans le cache
        self.cache.load_all().await?;

        // Charger les configurations depuis les fichiers utilisateurs
        let users_dir = PathBuf::from("ftp-data/users");
        if let Ok(user_configs) =
            crate::core_quota::user_file_parser::load_user_quota_configs(&users_dir)
        {
            let mut user_config = self.user_config.lock().await;
            for (username, (quota, ratio)) in user_configs {
                if let Some(q) = quota {
                    user_config.set_user_quota(&username, q);
                }
                if let Some(r) = ratio {
                    user_config.set_user_ratio(&username, &r);
                }
            }
        }

        Ok(())
    }

    /// Sauvegarde les données de quota dans les fichiers
    pub async fn save(&self) -> Result<(), QuotaError> {
        self.cache.save_all().await
    }

    /// Obtient ou crée un quota pour un utilisateur
    pub async fn get_or_create_user_quota(
        &self,
        username: &str,
        base_dir: PathBuf,
    ) -> Result<UserQuota, QuotaError> {
        // Vérifier si l'utilisateur a un quota spécifique
        if let Some(quota_bytes) = self.user_config.lock().await.get_user_quota(username) {
            let quota = UserQuota::new(username, quota_bytes, base_dir);
            self.cache
                .update_user_quota(username, quota.clone())
                .await?;
            return Ok(quota);
        }

        // Sinon, utiliser le quota par défaut
        self.cache
            .get_or_create_user_quota(username, base_dir)
            .await
    }

    /// Obtient ou crée un ratio pour un utilisateur
    pub async fn get_or_create_user_ratio(&self, username: &str) -> Result<UserRatio, QuotaError> {
        // Vérifier si l'utilisateur a un ratio spécifique
        if let Some(ratio_str) = self.user_config.lock().await.get_user_ratio(username) {
            let ratio = UserRatio::new(username, &ratio_str)?;
            self.cache
                .update_user_ratio(username, ratio.clone())
                .await?;
            return Ok(ratio);
        }

        // Sinon, utiliser le ratio par défaut
        self.cache.get_or_create_user_ratio(username).await
    }

    /// Vérifie si un utilisateur peut uploader un fichier
    pub async fn check_upload(
        &self,
        username: &str,
        base_dir: PathBuf,
        file_size: u64,
    ) -> Result<(), QuotaError> {
        if !self.config.enable_quota && !self.config.enable_ratio {
            return Ok(());
        }

        // Vérification du quota
        if self.config.enable_quota {
            let quota = self.get_or_create_user_quota(username, base_dir).await?;
            quota.check_quota(file_size)?;
        }

        Ok(())
    }

    /// Vérifie si un utilisateur peut télécharger un fichier
    pub async fn check_download(&self, username: &str, file_size: u64) -> Result<u64, QuotaError> {
        if !self.config.enable_ratio {
            return Ok(file_size);
        }

        let ratio = self.get_or_create_user_ratio(username).await?;
        ratio.check_download(file_size)
    }

    /// Met à jour les statistiques après un upload
    pub async fn record_upload(&self, username: &str, bytes: u64) -> Result<(), QuotaError> {
        // Mettre à jour le quota
        if self.config.enable_quota {
            let mut quota = self
                .cache
                .get_or_create_user_quota(username, PathBuf::from("/"))
                .await?;
            quota.update_used_bytes(bytes)?;
            self.cache.update_user_quota(username, quota).await?;
        }

        // Mettre à jour le ratio
        if self.config.enable_ratio {
            let mut ratio = self.cache.get_or_create_user_ratio(username).await?;
            ratio.update_uploaded(bytes);
            self.cache.update_user_ratio(username, ratio).await?;
        }

        // Mettre à jour les statistiques
        self.cache.update_user_stats(username, bytes, true).await?;

        Ok(())
    }

    /// Met à jour les statistiques après un download
    pub async fn record_download(&self, username: &str, bytes: u64) -> Result<(), QuotaError> {
        // Vérifier et mettre à jour le ratio
        if self.config.enable_ratio {
            let mut ratio = self.cache.get_or_create_user_ratio(username).await?;
            ratio.update_downloaded(bytes)?;
            self.cache.update_user_ratio(username, ratio).await?;
        }

        // Mettre à jour les statistiques
        self.cache.update_user_stats(username, bytes, false).await?;

        Ok(())
    }

    /// Obtient les informations de quota pour un utilisateur
    pub async fn get_quota_info(
        &self,
        username: &str,
        base_dir: PathBuf,
    ) -> Result<String, QuotaError> {
        let quota = self.get_or_create_user_quota(username, base_dir).await?;
        Ok(quota.format_quota())
    }

    /// Obtient les informations de ratio pour un utilisateur
    pub async fn get_ratio_info(&self, username: &str) -> Result<String, QuotaError> {
        let ratio = self.get_or_create_user_ratio(username).await?;
        Ok(ratio.format_ratio())
    }

    /// Obtient les statistiques de transfert pour un utilisateur
    pub async fn get_transfer_stats(&self, username: &str) -> Option<String> {
        // Pour les statistiques, nous devons accéder directement au cache
        let stats_manager = self.cache.stats_manager.read().await;
        stats_manager.get_stats(username).map(|s| s.format_stats())
    }
}
