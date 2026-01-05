// Commande SITE QUOTA - inspiré de glFTPd

use crate::core_quota::manager::QuotaManager;
use crate::{session::Session, Config};
use crate::core_ftpcommand::site::helper::{respond_with_error, respond_with_success};
use log::{info, warn};
use std::sync::Arc;
use tokio::net::TcpStream;
use tokio::sync::Mutex;

/// Gère la commande SITE QUOTA
/// Affiche les informations de quota pour l'utilisateur actuel

pub async fn handle_site_quota_command(
    writer: Arc<Mutex<TcpStream>>,
    _config: Arc<Config>,
    session: Arc<Mutex<Session>>,
    args: Vec<String>,
    quota_manager: Option<Arc<QuotaManager>>,
) -> Result<(), std::io::Error> {
    info!("Handling SITE QUOTA command");

    // Validation des arguments - cette commande ne prend pas d'arguments
    if !args.is_empty() {
        warn!("SITE QUOTA command does not accept arguments");
        respond_with_error(&writer, b"501 SITE QUOTA does not accept arguments.\r\n").await?;
        return Ok(());
    }

    let session = session.lock().await;
    let username = session
        .username
        .clone()
        .unwrap_or_else(|| "anonymous".to_string());
    let base_dir = session.base_path.clone();

    if let Some(quota_mgr) = quota_manager {
        match quota_mgr.get_quota_info(&username, base_dir).await {
            Ok(quota_info) => {
                let response = format!("200-Quota information for {}:\r\n", username);
                let mut writer = writer.lock().await;
                writer.write_all(response.as_bytes()).await?;

                let info_response = format!(" {}\r\n", quota_info);
                writer.write_all(info_response.as_bytes()).await?;

                respond_with_success(&writer, b"200 Quota command successful.\r\n").await?;
                info!("Sent quota info for user {}: {}", username, quota_info);
            }
            Err(e) => {
                warn!("Failed to get quota info for user {}: {}", username, e);
                respond_with_error(&writer, b"550 Failed to retrieve quota information.\r\n").await?;
            }
        }
    } else {
        respond_with_error(&writer, b"550 Quota system is not enabled.\r\n").await?;
    }

    Ok(())
}
