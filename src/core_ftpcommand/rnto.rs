use crate::core_ftpcommand::utils::{sanitize_input, send_response};
use crate::core_network::Session;
use crate::Config;
use anyhow::Result;
use log::{error, info, warn};
use std::path::PathBuf;
use std::sync::Arc;
use tokio::fs;
use tokio::net::TcpStream;
use tokio::sync::Mutex;

/// Handles the RNTO (Rename To) FTP command.
///
/// This function renames a file or directory on the server.
/// The function ensures the new name is within the allowed chroot area,
/// sanitizes inputs to prevent directory traversal attacks, and sends appropriate
/// responses back to the FTP client.
///
/// # Arguments
///
/// * `writer` - A shared, locked TCP stream for writing responses to the client.
/// * `config` - A shared server configuration.
/// * `session` - A shared, locked session containing the user's current state.
/// * `arg` - The new name of the file or directory.
///
/// # Returns
///
/// Result<(), std::io::Error> indicating the success or failure of the operation.
pub async fn handle_rnto_command(
    writer: Arc<Mutex<TcpStream>>,
    _config: Arc<Config>,
    session: Arc<Mutex<Session>>,
    arg: String,
) -> Result<(), std::io::Error> {
    // Sanitize the input argument to prevent directory traversal attacks.
    let sanitized_arg = sanitize_input(&arg);
    info!("Received RNTO command with argument: {}", sanitized_arg);

    // Retrieve the path of the file or directory to be renamed from the session.
    let old_path = {
        let session = session.lock().await;
        match &session.rename_from {
            Some(path) => path.clone(),
            None => {
                warn!("RNTO command issued without preceding RNFR command.");
                send_response(&writer, b"503 Bad sequence of commands.\r\n").await?;
                return Ok(());
            }
        }
    };
    info!("Old path to rename from: {:?}", old_path);

    // Construct the new path of the file or directory.
    let new_path = {
        let session = session.lock().await;
        session
            .base_path
            .join(&session.current_dir)
            .join(&sanitized_arg)
    };
    info!("New path to rename to: {:?}", new_path);

    // Canonicalize the new path to ensure it's within the chroot directory.
    let base_path = {
        let session = session.lock().await;
        session.base_path.clone()
    };
    let resolved_new_path = new_path.canonicalize().unwrap_or_else(|_| new_path.clone());

    // Check if the resolved new path is within the chroot directory.
    if !resolved_new_path.starts_with(&base_path) {
        error!(
            "Resolved new path is outside of the allowed area: {:?}",
            resolved_new_path
        );
        send_response(&writer, b"550 Path is outside of the allowed area.\r\n").await?;
        return Ok(());
    }

    // Attempt to rename the file or directory.
    match fs::rename(&old_path, &resolved_new_path).await {
        Ok(_) => {
            // Send success response if the file or directory was renamed successfully.
            info!(
                "File or directory renamed successfully from {:?} to {:?}",
                old_path, resolved_new_path
            );
            send_response(&writer, b"250 File or directory renamed successfully.\r\n").await?;
        }
        Err(e) => {
            // Send failure response if there was an error renaming the file or directory.
            error!(
                "Failed to rename file or directory from {:?} to {:?}: {}",
                old_path, resolved_new_path, e
            );
            send_response(&writer, b"550 Failed to rename file or directory.\r\n").await?;
        }
    }

    Ok(())
}
