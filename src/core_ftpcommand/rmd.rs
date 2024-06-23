use crate::core_network::Session;
use crate::helpers::{sanitize_input, send_response};
use crate::Config;
use anyhow::Result;
use log::{error, info, warn};
use std::path::PathBuf;
use std::sync::Arc;
use tokio::fs;
use tokio::net::TcpStream;
use tokio::sync::Mutex;

/// Handles the RMD (Remove Directory) FTP command.
///
/// This function deletes a directory on the server within the user's current directory.
/// The function ensures the directory is within the allowed chroot area, sanitizes inputs to
/// prevent directory traversal attacks, and sends appropriate responses back to the FTP client.
///
/// # Arguments
///
/// * `writer` - A shared, locked TCP stream for writing responses to the client.
/// * `config` - A shared server configuration.
/// * `session` - A shared, locked session containing the user's current state.
/// * `arg` - The directory name to delete.
///
/// # Returns
///
/// Result<(), std::io::Error> indicating the success or failure of the operation.
pub async fn handle_rmd_command(
    writer: Arc<Mutex<TcpStream>>,
    _config: Arc<Config>,
    session: Arc<Mutex<Session>>,
    arg: String,
) -> Result<(), std::io::Error> {
    // Sanitize the input argument to prevent directory traversal attacks.
    let sanitized_arg = sanitize_input(&arg);
    info!("Received RMD command with argument: {}", sanitized_arg);

    // Construct the directory path within the user's current directory.
    let dir_path = {
        // Lock the session to get the current directory.
        let session = session.lock().await;
        session
            .base_path
            .join(&session.current_dir)
            .join(&sanitized_arg)
    };
    info!("Constructed directory path: {:?}", dir_path);

    // Canonicalize the chroot directory to resolve any symbolic links or relative paths.
    let chroot_dir = PathBuf::from(&session.lock().await.base_path).canonicalize()?;
    // Canonicalize the directory path to ensure it's within the chroot directory.
    let resolved_path = dir_path.canonicalize().unwrap_or_else(|_| dir_path.clone());

    // Check if the resolved path is within the chroot directory.
    if !resolved_path.starts_with(&chroot_dir) {
        error!("Path is outside of the allowed area: {:?}", resolved_path);
        send_response(&writer, b"550 Path is outside of the allowed area.\r\n").await?;
        return Ok(());
    }

    // Check if the directory exists.
    if !resolved_path.exists() {
        warn!("Directory does not exist: {:?}", resolved_path);
        send_response(&writer, b"550 Directory does not exist.\r\n").await?;
        return Ok(());
    }

    // Attempt to remove the directory.
    match fs::remove_dir(&resolved_path).await {
        Ok(_) => {
            // Send success response if the directory was deleted successfully.
            info!("Directory removed successfully: {:?}", resolved_path);
            send_response(
                &writer,
                format!("250 \"{}\" directory removed.\r\n", sanitized_arg).as_bytes(),
            )
            .await?;
        }
        Err(e) => {
            // Send failure response if there was an error deleting the directory.
            error!(
                "Failed to remove directory: {:?}, error: {}",
                resolved_path, e
            );
            send_response(&writer, b"550 Failed to remove directory.\r\n").await?;
        }
    }

    Ok(())
}
