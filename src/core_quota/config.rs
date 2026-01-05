// Configuration des quotas et ratios - inspiré de glFTPd

use crate::core_quota::error::QuotaError;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

/// Configuration globale des quotas
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuotaConfig {
    /// Quota par défaut pour les nouveaux utilisateurs (en octets)
    pub default_quota: u64,

    /// Ratio par défaut pour les nouveaux utilisateurs (format "upload:download")
    pub default_ratio: String,

    /// Fichier de stockage des quotas utilisateurs
    pub quota_storage_file: PathBuf,

    /// Fichier de stockage des ratios utilisateurs
    pub ratio_storage_file: PathBuf,

    /// Fichier de stockage des statistiques de transfert
    pub stats_storage_file: PathBuf,

    /// Activation du système de quota
    pub enable_quota: bool,

    /// Activation du système de ratio
    pub enable_ratio: bool,
}

impl Default for QuotaConfig {
    fn default() -> Self {
        Self {
            default_quota: 10737418240, // 10 GB
            default_ratio: "1:1".to_string(),
            quota_storage_file: PathBuf::from("data/quotas.json"),
            ratio_storage_file: PathBuf::from("data/ratios.json"),
            stats_storage_file: PathBuf::from("data/transfer_stats.json"),
            enable_quota: true,
            enable_ratio: true,
        }
    }
}

/// Configuration des quotas par groupe
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroupQuotaConfig {
    /// Quotas par groupe
    pub group_quotas: HashMap<String, u64>,

    /// Ratios par groupe
    pub group_ratios: HashMap<String, String>,
}

impl GroupQuotaConfig {
    pub fn new() -> Self {
        Self {
            group_quotas: HashMap::new(),
            group_ratios: HashMap::new(),
        }
    }

    /// Ajoute ou met à jour un quota de groupe
    pub fn set_group_quota(&mut self, groupname: &str, quota: u64) {
        self.group_quotas.insert(groupname.to_string(), quota);
    }

    /// Obtient le quota pour un groupe
    pub fn get_group_quota(&self, groupname: &str) -> Option<u64> {
        self.group_quotas.get(groupname).copied()
    }

    /// Ajoute ou met à jour un ratio de groupe
    pub fn set_group_ratio(&mut self, groupname: &str, ratio: &str) {
        self.group_ratios
            .insert(groupname.to_string(), ratio.to_string());
    }

    /// Obtient le ratio pour un groupe
    pub fn get_group_ratio(&self, groupname: &str) -> Option<String> {
        self.group_ratios.get(groupname).cloned()
    }
}

/// Configuration des quotas par utilisateur
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserQuotaConfig {
    /// Quotas spécifiques par utilisateur
    pub user_quotas: HashMap<String, u64>,

    /// Ratios spécifiques par utilisateur
    pub user_ratios: HashMap<String, String>,
}

impl UserQuotaConfig {
    pub fn new() -> Self {
        Self {
            user_quotas: HashMap::new(),
            user_ratios: HashMap::new(),
        }
    }

    /// Ajoute ou met à jour un quota utilisateur
    pub fn set_user_quota(&mut self, username: &str, quota: u64) {
        self.user_quotas.insert(username.to_string(), quota);
    }

    /// Obtient le quota pour un utilisateur
    pub fn get_user_quota(&self, username: &str) -> Option<u64> {
        self.user_quotas.get(username).copied()
    }

    /// Ajoute ou met à jour un ratio utilisateur
    pub fn set_user_ratio(&mut self, username: &str, ratio: &str) {
        self.user_ratios
            .insert(username.to_string(), ratio.to_string());
    }

    /// Obtient le ratio pour un utilisateur
    pub fn get_user_ratio(&self, username: &str) -> Option<String> {
        self.user_ratios.get(username).cloned()
    }
}

/// Parser de configuration pour les quotas/ratios
pub struct QuotaConfigParser;

impl QuotaConfigParser {
    /// Parse une configuration de quota/ratio depuis un fichier
    /// Format inspiré de glFTPd mais adapté à TOML
    pub fn parse_from_toml(
        toml_content: &str,
    ) -> Result<(QuotaConfig, GroupQuotaConfig, UserQuotaConfig), QuotaError> {
        let config: QuotaConfig = toml::from_str(toml_content)
            .map_err(|e| QuotaError::InvalidQuotaConfig(e.to_string()))?;

        // Pour l'instant, nous retournons des configurations de groupe et utilisateur vides
        // Elles seront remplies par le système de fichiers utilisateur
        let group_config = GroupQuotaConfig::new();
        let user_config = UserQuotaConfig::new();

        Ok((config, group_config, user_config))
    }

    /// Parse une configuration depuis un fichier
    pub fn parse_from_file(
        file_path: &str,
    ) -> Result<(QuotaConfig, GroupQuotaConfig, UserQuotaConfig), QuotaError> {
        let content = std::fs::read_to_string(file_path)
            .map_err(|e| QuotaError::QuotaReadError(e.to_string()))?;

        Self::parse_from_toml(&content)
    }
}
