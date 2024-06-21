use std::sync::Arc;
use tokio::net::TcpStream;
use tokio::sync::Mutex;
use crate::Config;
use std::path::PathBuf;
use tokio::fs;
use crate::core_network::Session;
use crate::core_ftpcommand::utils::{sanitize_input, construct_path, send_response};
use anyhow::Result;
use log::{info, warn, error};

/// Handles the MKD (Make Directory) FTP command.
///
/// This function creates a new directory on the server within the user's current directory.
/// The function ensures the new directory is within the allowed chroot area, sanitizes inputs to
/// prevent directory traversal attacks, and sends appropriate responses back to the FTP client.
///
/// # Arguments
///
/// * `writer` - A shared, locked TCP stream for writing responses to the client.
/// * `config` - A shared server configuration.
/// * `session` - A shared, locked session containing the user's current state.
/// * `arg` - The directory name to create.
///
/// # Returns
///
/// Result<(), std::io::Error> indicating the success or failure of the operation.
pub async fn handle_mkd_command(
    writer: Arc<Mutex<TcpStream>>,
    config: Arc<Config>,
    session: Arc<Mutex<Session>>,
    arg: String,
) -> Result<(), std::io::Error> {
    // Sanitize the input argument to prevent directory traversal attacks.
    let sanitized_arg = sanitize_input(&arg);
    info!("Received MKD command with argument: {}", sanitized_arg);

    // Construct the new directory path within the user's current directory.
    let dir_path = {
        // Lock the session to get the current directory.
        let session = session.lock().await;
        construct_path(&config, &session.current_dir, &sanitized_arg)
    };

    // Log the constructed directory path
    info!("Constructed directory path: {:?}", dir_path);

    // Canonicalize the chroot directory to resolve any symbolic links or relative paths.
    let chroot_dir = PathBuf::from(&config.server.chroot_dir).canonicalize()?;
    // Canonicalize the new directory path to ensure it's within the chroot directory.
    let resolved_path = dir_path.canonicalize().unwrap_or_else(|_| dir_path.clone());

    // Check if the resolved path is within the chroot directory.
    if !resolved_path.starts_with(&chroot_dir) {
        error!("Path is outside of the allowed area: {:?}", resolved_path);
        send_response(&writer, b"550 Path is outside of the allowed area.\r\n").await?;
        return Ok(());
    }

    // Check if the directory already exists.
    if resolved_path.exists() {
        warn!("Directory already exists: {:?}", resolved_path);
        send_response(&writer, b"550 Directory already exists.\r\n").await?;
        return Ok(());
    }

    // Attempt to create the directory.
    match fs::create_dir_all(&resolved_path).await {
        Ok(_) => {
            // Send success response if the directory was created successfully.
            info!("Directory created successfully: {:?}", resolved_path);
            send_response(&writer, format!("257 \"{}\" directory created.\r\n", sanitized_arg).as_bytes()).await?;
        },
        Err(e) => {
            // Send failure response if there was an error creating the directory.
            error!("Failed to create directory: {:?}, error: {}", resolved_path, e);
            send_response(&writer, b"550 Failed to create directory.\r\n").await?;
        }
    }

    Ok(())
}
