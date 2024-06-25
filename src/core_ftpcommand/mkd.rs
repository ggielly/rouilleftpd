use crate::helpers::{sanitize_input, send_response};
use crate::session::Session;
use crate::Config;
use anyhow::Result;
use log::{error, info, warn};
use std::sync::Arc;
use tokio::fs;
use tokio::net::TcpStream;
use tokio::sync::Mutex;

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
    _config: Arc<Config>,
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
        let base_path = session.base_path.clone();
        let current_dir = session.current_dir.clone();
        let dir_path = base_path
            .join(current_dir.trim_start_matches('/'))
            .join(&sanitized_arg);

        info!("base_path: {:?}", base_path);
        info!("current_dir: {:?}", current_dir);
        info!("dir_path: {:?}", dir_path);

        dir_path
    };

    // Log the constructed directory path
    info!("Constructed directory path: {:?}", dir_path);

    // Canonicalize the base path.
    let canonical_base_path = dir_path.parent().unwrap().canonicalize()?;

    // Check if the resolved path is within the chroot directory.
    if !canonical_base_path.starts_with(&canonical_base_path) {
        error!(
            "Path is outside of the allowed area: {:?}",
            canonical_base_path
        );
        send_response(&writer, b"550 Path is outside of the allowed area.\r\n").await?;
        return Ok(());
    }

    // Check if the directory already exists.
    if dir_path.exists() {
        warn!("Directory already exists: {:?}", dir_path);
        send_response(&writer, b"550 Directory already exists.\r\n").await?;
        return Ok(());
    }

    // Attempt to create the directory.
    match fs::create_dir_all(&dir_path).await {
        Ok(_) => {
            // Send success response if the directory was created successfully.
            info!("Directory created successfully: {:?}", dir_path);
            send_response(
                &writer,
                format!("257 \"{}\" directory created.\r\n", sanitized_arg).as_bytes(),
            )
            .await?;
        }
        Err(e) => {
            // Send failure response if there was an error creating the directory.
            error!("Failed to create directory: {:?}, error: {}", dir_path, e);
            send_response(&writer, b"550 Failed to create directory.\r\n").await?;
        }
    }

    Ok(())
}
