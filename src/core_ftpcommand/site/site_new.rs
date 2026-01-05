// Commande SITE NEW - Notification de nouveaux fichiers
// Inspiré de glFTPd

use crate::core_quota::manager::QuotaManager;
use crate::{session::Session, Config};
use crate::core_ftpcommand::site::helper::{respond_with_error, respond_with_success};
use log::{info, warn};
use std::sync::Arc;
use tokio::net::TcpStream;
use tokio::sync::Mutex;

/// Gère la commande SITE NEW
/// Notifie lorsque de nouveaux fichiers sont disponibles dans un répertoire

pub async fn handle_site_new_command(
    writer: Arc<Mutex<TcpStream>>,
    config: Arc<Config>,
    session: Arc<Mutex<Session>>,
    args: Vec<String>,
    _quota_manager: Option<Arc<QuotaManager>>,
) -> Result<(), std::io::Error> {
    info!("Handling SITE NEW command with args: {:?}", args);

    let session = session.lock().await;
    let base_path = session.base_path.clone();

    // Déterminer le répertoire à surveiller
    let dir_path = if args.is_empty() {
        base_path.join("incoming") // Répertoire par défaut
    } else {
        base_path.join(&args.join(" "))
    };

    // Vérifier que le chemin est valide
    if !dir_path.starts_with(&base_path) {
        warn!("Path traversal attempt in SITE NEW: {:?}", dir_path);
        respond_with_error(&writer, b"550 Path is outside of the allowed area.\r\n").await?;
        return Ok(());
    }

    // Vérifier que le répertoire existe
    if !dir_path.exists() {
        warn!("Directory not found for SITE NEW: {:?}", dir_path);
        respond_with_error(&writer, b"550 Directory not found.\r\n").await?;
        return Ok(());
    }

    // En-tête de la réponse
    respond_with_success(&writer, format!("200-NEW command for directory: {:?}\r\n", dir_path).as_bytes()).await?;

    // Lister les fichiers récents (à implémenter complètement)
    match std::fs::read_dir(&dir_path) {
        Ok(entries) => {
            let mut file_count = 0;
            for entry in entries.flatten() {
                if file_count >= 10 {
                    respond_with_success(&writer, b"  ... (more files)\r\n").await?;
                    break;
                }

                if let Ok(metadata) = entry.metadata() {
                    if metadata.is_file() {
                        let file_name = entry.file_name();
                        let file_name_str = file_name.to_string_lossy();
                        respond_with_success(&writer, format!("  {}\r\n", file_name_str).as_bytes()).await?;
                        file_count += 1;
                    }
                }
            }

            if file_count == 0 {
                respond_with_success(&writer, b"  No new files found.\r\n").await?;
            }
        }
        Err(e) => {
            warn!("Failed to read directory for SITE NEW: {}", e);
            respond_with_error(&writer, b"  Error reading directory.\r\n").await?;
        }
    }

    respond_with_success(&writer, b"200 NEW command successful.\r\n").await?;

    Ok(())
}
