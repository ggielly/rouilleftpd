// Commande SITE IDLE - Gestion de l'inactivité
// Inspiré de glFTPd

use crate::core_quota::manager::QuotaManager;
use crate::{session::Session, Config};
use crate::core_ftpcommand::site::helper::{respond_with_error, respond_with_success};
use log::info;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::net::TcpStream;
use tokio::sync::Mutex;

/// Gère la commande SITE IDLE
/// Affiche le temps d'inactivité de la session actuelle

pub async fn handle_site_idle_command(
    writer: Arc<Mutex<TcpStream>>,
    _config: Arc<Config>,
    session: Arc<Mutex<Session>>,
    _args: Vec<String>,
    _quota_manager: Option<Arc<QuotaManager>>,
) -> Result<(), std::io::Error> {
    info!("Handling SITE IDLE command");

    let session = session.lock().await;
    let username = session
        .username
        .clone()
        .unwrap_or_else(|| "anonymous".to_string());

    // Pour l'instant, nous simulons le temps d'inactivité
    // Dans une implémentation complète, nous suivrions le dernier timestamp d'activité
    let start_time = SystemTime::now();
    let since_epoch = start_time
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards");

    // Simuler un temps d'inactivité (en secondes)
    let idle_time = since_epoch.as_secs() % 300; // 0-300 secondes pour la démo

    // En-tête de la réponse
    respond_with_success(&writer, b"200-IDLE command:\r\n").await?;

    // Afficher le temps d'inactivité
    respond_with_success(&writer, format!("  User: {}\r\n", username).as_bytes()).await?;
    respond_with_success(&writer, format!("  Idle time: {} seconds\r\n", idle_time).as_bytes()).await?;

    // Statut
    if idle_time > 180 {
        respond_with_success(&writer, b"  Status: About to timeout\r\n").await?;
    } else if idle_time > 120 {
        respond_with_success(&writer, b"  Status: Idle\r\n").await?;
    } else {
        respond_with_success(&writer, b"  Status: Active\r\n").await?;
    }

    respond_with_success(&writer, b"200 IDLE command successful.\r\n").await?;

    Ok(())
}
