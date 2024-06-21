use crate::core_ftpcommand::utils::{construct_path, sanitize_input, send_response};
use crate::core_network::Session;
use crate::Config;
use anyhow::Result;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::fs;
use tokio::io::AsyncWriteExt;
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
    config: Arc<Config>,
    session: Arc<Mutex<Session>>,
    arg: String,
) -> Result<(), std::io::Error> {
    // Sanitize the input argument to prevent directory traversal attacks.
    let sanitized_arg = sanitize_input(&arg);

    // Retrieve the path of the file or directory to be renamed from the session.
    let old_path = {
        let session = session.lock().await;
        match &session.rename_from {
            Some(path) => path.clone(),
            None => {
                send_response(&writer, b"503 Bad sequence of commands.\r\n").await?;
                return Ok(());
            }
        }
    };

    // Construct the new path of the file or directory.
    let new_path = {
        let session = session.lock().await;
        construct_path(&config, &session.current_dir, &sanitized_arg)
    };

    // Canonicalize the chroot directory to resolve any symbolic links or relative paths.
    let chroot_dir = PathBuf::from(&config.server.chroot_dir).canonicalize()?;
    // Canonicalize the new path to ensure it's within the chroot directory.
    let resolved_new_path = new_path.canonicalize().unwrap_or_else(|_| new_path.clone());

    // Check if the resolved new path is within the chroot directory.
    if !resolved_new_path.starts_with(&chroot_dir) {
        send_response(&writer, b"550 Path is outside of the allowed area.\r\n").await?;
        return Ok(());
    }

    // Attempt to rename the file or directory.
    match fs::rename(&old_path, &resolved_new_path).await {
        Ok(_) => {
            // Send success response if the file or directory was renamed successfully.
            send_response(&writer, b"250 File or directory renamed successfully.\r\n").await?;
        }
        Err(_) => {
            // Send failure response if there was an error renaming the file or directory.
            send_response(&writer, b"550 Failed to rename file or directory.\r\n").await?;
        }
    }

    Ok(())
}
