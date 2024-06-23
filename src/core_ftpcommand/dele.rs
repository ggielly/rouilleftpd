use crate::session::Session;
use crate::helpers::{sanitize_input, send_response};
use crate::Config;
use anyhow::Result;
use log::{error, info, warn};
use std::path::PathBuf;
use std::sync::Arc;
use tokio::fs;
use tokio::net::TcpStream;
use tokio::sync::Mutex;

/// Handles the DELE (Delete File) FTP command.
///
/// This function deletes a file on the server within the user's current directory.
/// The function ensures the file is within the allowed chroot area, sanitizes inputs to
/// prevent directory traversal attacks, and sends appropriate responses back to the FTP client.
///
/// # Arguments
///
/// * `writer` - A shared, locked TCP stream for writing responses to the client.
/// * `config` - A shared server configuration.
/// * `session` - A shared, locked session containing the user's current state.
/// * `arg` - The file name to delete.
///
/// # Returns
///
/// Result<(), std::io::Error> indicating the success or failure of the operation.
pub async fn handle_dele_command(
    writer: Arc<Mutex<TcpStream>>,
    config: Arc<Config>,
    session: Arc<Mutex<Session>>,
    arg: String,
) -> Result<(), std::io::Error> {
    // Sanitize the input argument to prevent directory traversal attacks.
    let sanitized_arg = sanitize_input(&arg);
    info!("Received DELE command with argument: {}", sanitized_arg);

    // Construct the file path within the user's current directory.
    let file_path = {
        // Lock the session to get the current directory.
        let session = session.lock().await;
        session
            .base_path
            .join(&session.current_dir)
            .join(&sanitized_arg)
    };
    info!("Constructed file path: {:?}", file_path);

    // Canonicalize the chroot directory to resolve any symbolic links or relative paths.
    let chroot_dir = PathBuf::from(&config.server.chroot_dir).canonicalize()?;
    // Canonicalize the file path to ensure it's within the chroot directory.
    let resolved_path = file_path
        .canonicalize()
        .unwrap_or_else(|_| file_path.clone());

    // Check if the resolved path is within the chroot directory.
    if !resolved_path.starts_with(&chroot_dir) {
        error!("Path is outside of the allowed area: {:?}", resolved_path);
        send_response(&writer, b"550 Path is outside of the allowed area.\r\n").await?;
        return Ok(());
    }

    // Check if the file exists.
    if !resolved_path.exists() {
        warn!("File does not exist: {:?}", resolved_path);
        send_response(&writer, b"550 File does not exist.\r\n").await?;
        return Ok(());
    }

    // Attempt to delete the file.
    match fs::remove_file(&resolved_path).await {
        Ok(_) => {
            // Send success response if the file was deleted successfully.
            info!("File deleted successfully: {:?}", resolved_path);
            send_response(
                &writer,
                format!("250 \"{}\" file deleted.\r\n", sanitized_arg).as_bytes(),
            )
            .await?;
        }
        Err(e) => {
            // Send failure response if there was an error deleting the file.
            error!("Failed to delete file: {:?}, error: {}", resolved_path, e);
            send_response(&writer, b"550 Failed to delete file.\r\n").await?;
        }
    }

    Ok(())
}
