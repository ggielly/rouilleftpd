// Commande SITE GROUP - Gestion des groupes d'utilisateurs
// Inspiré de glFTPd

use crate::core_quota::manager::QuotaManager;
use crate::{session::Session, Config};
use crate::core_ftpcommand::site::helper::{respond_with_error, respond_with_success};
use log::{info, warn};
use std::sync::Arc;
use tokio::net::TcpStream;
use tokio::sync::Mutex;

/// Gère la commande SITE GROUP
/// Permet d'afficher et de modifier les informations de groupe d'un utilisateur

pub async fn handle_site_group_command(
    writer: Arc<Mutex<TcpStream>>,
    _config: Arc<Config>,
    session: Arc<Mutex<Session>>,
    args: Vec<String>,
    _quota_manager: Option<Arc<QuotaManager>>,
) -> Result<(), std::io::Error> {
    info!("Handling SITE GROUP command with args: {:?}", args);

    let session = session.lock().await;
    let username = session
        .username
        .clone()
        .unwrap_or_else(|| "anonymous".to_string());

    // Validation des arguments
    if args.len() > 1 {
        warn!("Too many arguments for SITE GROUP command");
        respond_with_error(&writer, b"501 Syntax error in parameters or arguments.\r\n").await?;
        respond_with_error(&writer, b"501 Usage: SITE GROUP [groupname]\r\n").await?;
        return Ok(());
    }

    // En-tête de la réponse
    let response = if args.is_empty() {
        format!("200-Group information for {}:\r\n", username)
    } else {
        format!("200-Group command processed for {}:\r\n", username)
    };
    respond_with_success(&writer, response.as_bytes()).await?;

    // Ajouter des informations de groupe (à implémenter)
    let group_info = if args.is_empty() {
        "  Group: users (default)\r\n"
    } else {
        format!("  Group: {} (requested)\r\n", args[0])
    };
    respond_with_success(&writer, group_info.as_bytes()).await?;

    // Note sur l'implémentation future
    respond_with_success(&writer, b"  Note: Full group management will be implemented in future versions\r\n").await?;

    respond_with_success(&writer, b"200 GROUP command successful.\r\n").await?;

    Ok(())
}
