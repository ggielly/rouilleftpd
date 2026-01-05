use crate::core_network::network;
use crate::core_quota::config::{
    GroupQuotaConfig, QuotaConfig as CoreQuotaConfig, UserQuotaConfig,
};
use crate::core_quota::manager::QuotaManager;
use crate::helpers::log_config;
use crate::ipc::Ipc;
use crate::session::Session;
use crate::Config;
use anyhow::Result;
use log::{error, info};
use std::path::PathBuf;
use std::sync::Arc;

/// Runs the FTP server with the provided configuration and IPC key.
///
/// This function initializes the server configuration and starts the FTP server,
/// logging significant steps and potential issues.
///
/// # Arguments
///
/// * `config` - The server configuration.
/// * `ipc` - The IPC instance for inter-process communication.
///
/// # Returns
///
/// Result<(), anyhow::Error> indicating the success or failure of the operation.
pub async fn run(config: Config, ipc: Arc<Ipc>) -> Result<()> {
    // Log each configuration option on a new line
    info!("Starting server with the following configuration:");
    log_config(&config);

    // Log IPC key separately as it's not part of the config struct
    info!("IPC Key: {:?}", ipc.ipc_key);

    // Initialize quota manager if configured
    let quota_manager = if let Some(quota_config) = &config.quota {
        info!("Initializing quota system");
        let core_quota_config = CoreQuotaConfig {
            default_quota: quota_config.default_quota.unwrap_or(10737418240),
            default_ratio: quota_config
                .default_ratio
                .clone()
                .unwrap_or_else(|| "1:1".to_string()),
            quota_storage_file: quota_config
                .quota_storage_file
                .clone()
                .unwrap_or_else(|| PathBuf::from("data/quotas.json")),
            ratio_storage_file: quota_config
                .ratio_storage_file
                .clone()
                .unwrap_or_else(|| PathBuf::from("data/ratios.json")),
            stats_storage_file: quota_config
                .stats_storage_file
                .clone()
                .unwrap_or_else(|| PathBuf::from("data/transfer_stats.json")),
            enable_quota: quota_config.enable_quota.unwrap_or(true),
            enable_ratio: quota_config.enable_ratio.unwrap_or(true),
        };

        let group_config = GroupQuotaConfig::new();
        let user_config = UserQuotaConfig::new();

        let manager = QuotaManager::new(core_quota_config, group_config, user_config);

        // Load existing quota data
        if let Err(e) = manager.load().await {
            error!("Failed to load quota data: {}", e);
        }

        Some(Arc::new(manager))
    } else {
        info!("Quota system disabled");
        None
    };

    // Start the FTP server
    match network::start_server(
        config.server.listen_port,
        Arc::new(config),
        ipc,
        quota_manager,
    )
    .await
    {
        Ok(_) => info!("Server started successfully."),
        Err(e) => {
            error!("Failed to start server: {}", e);
            return Err(e);
        }
    }

    Ok(())
}

pub fn initialize_session(config: &Config) -> Session {
    let base_path = PathBuf::from(&config.server.chroot_dir)
        .join(config.server.min_homedir.trim_start_matches('/'))
        .canonicalize()
        .unwrap();

    Session::new(base_path)
}
