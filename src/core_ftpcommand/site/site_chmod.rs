// Commande SITE CHMOD - Changement de permissions
// Inspiré de glFTPd

use crate::core_quota::manager::QuotaManager;
use crate::{session::Session, Config};
use crate::core_ftpcommand::site::helper::{respond_with_error, respond_with_success};
use log::{info, warn};
use std::sync::Arc;
use tokio::net::TcpStream;
use tokio::sync::Mutex;

/// Gère la commande SITE CHMOD
/// Permet de changer les permissions des fichiers et répertoires

pub async fn handle_site_chmod_command(
    writer: Arc<Mutex<TcpStream>>,
    config: Arc<Config>,
    session: Arc<Mutex<Session>>,
    args: Vec<String>,
    _quota_manager: Option<Arc<QuotaManager>>,
) -> Result<(), std::io::Error> {
    info!("Handling SITE CHMOD command with args: {:?}", args);

    if args.len() < 2 {
        warn!("Insufficient arguments for SITE CHMOD command");
        respond_with_error(&writer, b"501 Syntax error in parameters or arguments.\r\n").await?;
        respond_with_error(&writer, b"501 Usage: SITE CHMOD <mode> <file>\r\n").await?;
        return Ok(());
    }

    let mode_str = &args[0];
    let file_path_str = &args[1..].join(" ");

    // Parser le mode (octale)
    match u32::from_str_radix(mode_str, 8) {
        Ok(_) => {}
        Err(_) => {
            warn!("Invalid mode format: {}", mode_str);
            respond_with_error(&writer, b"501 Invalid mode format. Use octal notation (e.g., 644).\r\n").await?;
            return Ok(());
        }
    };

    let session = session.lock().await;
    let base_path = session.base_path.clone();

    // Construire le chemin complet en toute sécurité
    let file_path = base_path.join(file_path_str);

    // Vérifier que le chemin est dans la zone autorisée
    if !file_path.starts_with(&base_path) {
        warn!("Path traversal attempt: {:?}", file_path);
        respond_with_error(&writer, b"550 Path is outside of the allowed area.\r\n").await?;
        return Ok(());
    }

    // Vérifier que le fichier existe
    if !file_path.exists() {
        warn!("File not found: {:?}", file_path);
        respond_with_error(&writer, b"550 File not found.\r\n").await?;
        return Ok(());
    }

    // Changer les permissions
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mode = u32::from_str_radix(mode_str, 8).unwrap();
        match std::fs::set_permissions(&file_path, std::fs::Permissions::from_mode(mode)) {
            Ok(_) => {
                info!("Changed permissions of {:?} to {}", file_path, mode_str);
                respond_with_success(&writer, b"200 SITE CHMOD command successful.\r\n").await?;
            }
            Err(e) => {
                warn!("Failed to change permissions of {:?}: {}", file_path, e);
                respond_with_error(&writer, b"550 Failed to change permissions.\r\n").await?;
            }
        }
    }
    #[cfg(not(unix))]
    {
        warn!("CHMOD not supported on this platform for {:?}", file_path);
        let mut writer = writer.lock().await;
        writer
            .write_all(b"502 CHMOD command not supported on this platform.\r\n")
            .await?;
    }

    Ok(())
}
