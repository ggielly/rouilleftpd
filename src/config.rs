use crate::core_tls::tls_config::TlsConfig;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Deserialize, Serialize)]
pub struct ServerConfig {
    pub listen_port: u16,
    pub pasv_address: String,
    pub ipc_key: String,
    pub chroot_dir: String,
    pub min_homedir: String,
    pub upload_buffer_size: Option<usize>, // Optional to allow default value
    pub download_buffer_size: Option<usize>, // Optional to allow default value
    pub passwd_file: String,               // Path to the shadow (passwd) file
}

#[derive(Debug, Deserialize, Serialize)]
pub struct QuotaConfig {
    /// Quota par défaut pour les nouveaux utilisateurs (en octets)
    pub default_quota: Option<u64>,

    /// Ratio par défaut pour les nouveaux utilisateurs (format "upload:download")
    pub default_ratio: Option<String>,

    /// Fichier de stockage des quotas utilisateurs
    pub quota_storage_file: Option<PathBuf>,

    /// Fichier de stockage des ratios utilisateurs
    pub ratio_storage_file: Option<PathBuf>,

    /// Fichier de stockage des statistiques de transfert
    pub stats_storage_file: Option<PathBuf>,

    /// Activation du système de quota
    pub enable_quota: Option<bool>,

    /// Activation du système de ratio
    pub enable_ratio: Option<bool>,
}

impl Default for QuotaConfig {
    fn default() -> Self {
        Self {
            default_quota: Some(10737418240), // 10 GB
            default_ratio: Some("1:1".to_string()),
            quota_storage_file: Some(PathBuf::from("data/quotas.json")),
            ratio_storage_file: Some(PathBuf::from("data/ratios.json")),
            stats_storage_file: Some(PathBuf::from("data/transfer_stats.json")),
            enable_quota: Some(true),
            enable_ratio: Some(true),
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Config {
    pub server: ServerConfig,
    pub quota: Option<QuotaConfig>,
    pub tls: Option<TlsConfig>,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            listen_port: 21,
            pasv_address: String::from("0.0.0.0"),
            ipc_key: String::from("DEADBABE"),
            chroot_dir: String::from("/rouilleftpd"),
            min_homedir: String::from("/"),
            upload_buffer_size: Some(256 * 1024), // Default 256 KB
            download_buffer_size: Some(128 * 1024), // Default 128 KB

            passwd_file: String::from("/etc/passwd"),
        }
    }
}

impl Config {
    pub fn load_from_file(path: &str) -> Self {
        let config_str = std::fs::read_to_string(path).expect("Failed to read config file");
        let mut config: Config = toml::from_str(&config_str).expect("Failed to parse config file");

        // Set defaults if not specified
        if config.server.upload_buffer_size.is_none() {
            config.server.upload_buffer_size = Some(256 * 1024);
        }
        if config.server.download_buffer_size.is_none() {
            config.server.download_buffer_size = Some(128 * 1024);
        }

        // Set quota defaults if not specified
        if config.quota.is_none() {
            config.quota = Some(QuotaConfig::default());
        } else {
            let quota_config = config.quota.as_mut().unwrap();
            if quota_config.default_quota.is_none() {
                quota_config.default_quota = Some(10737418240); // 10 GB
            }
            if quota_config.default_ratio.is_none() {
                quota_config.default_ratio = Some("1:1".to_string());
            }
            if quota_config.quota_storage_file.is_none() {
                quota_config.quota_storage_file = Some(PathBuf::from("data/quotas.json"));
            }
            if quota_config.ratio_storage_file.is_none() {
                quota_config.ratio_storage_file = Some(PathBuf::from("data/ratios.json"));
            }
            if quota_config.stats_storage_file.is_none() {
                quota_config.stats_storage_file = Some(PathBuf::from("data/transfer_stats.json"));
            }
            if quota_config.enable_quota.is_none() {
                quota_config.enable_quota = Some(true);
            }
            if quota_config.enable_ratio.is_none() {
                quota_config.enable_ratio = Some(true);
            }
        }

        config
    }
}
