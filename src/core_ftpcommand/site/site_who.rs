// Commande SITE WHO - Liste des utilisateurs connectés
// Inspiré de glFTPd

use crate::core_quota::manager::QuotaManager;
use crate::{session::Session, Config};
use crate::core_ftpcommand::site::helper::{respond_with_error, respond_with_success};
use log::info;
use std::sync::Arc;
use tokio::net::TcpStream;
use tokio::sync::Mutex;

/// Gère la commande SITE WHO
/// Affiche la liste des utilisateurs actuellement connectés

pub async fn handle_site_who_command(
    writer: Arc<Mutex<TcpStream>>,
    _config: Arc<Config>,
    session: Arc<Mutex<Session>>,
    _args: Vec<String>,
    _quota_manager: Option<Arc<QuotaManager>>,
) -> Result<(), std::io::Error> {
    info!("Handling SITE WHO command");

    let session = session.lock().await;
    let username = session
        .username
        .clone()
        .unwrap_or_else(|| "anonymous".to_string());

    // Pour l'instant, nous retournons une réponse basique
    // Cette commande sera étendue pour afficher les utilisateurs connectés
    let mut writer = writer.lock().await;

    // En-tête de la réponse
    writer.write_all(b"200-WHO command:\r\n").await?;

    // Afficher l'utilisateur actuel
    writer
        .write_all(format!("  User: {} (current session)\r\n", username).as_bytes())
        .await?;

    // Note sur les utilisateurs connectés (à implémenter)
    writer
        .write_all(b"  Note: Full user list will be implemented in future versions\r\n")
        .await?;

    respond_with_success(&writer, b"200 WHO command successful.\r\n").await?;

    Ok(())
}
