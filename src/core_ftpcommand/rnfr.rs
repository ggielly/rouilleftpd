use crate::core_ftpcommand::utils::{sanitize_input, send_response};
use crate::session::Session;
use crate::Config;
use anyhow::Result;
use log::{error, info, warn};
use std::sync::Arc;
use tokio::net::TcpStream;
use tokio::sync::Mutex;

/// Handles the RNFR (Rename From) FTP command.
///
/// This function sets the file or directory to be renamed on the server.
/// The function ensures the file or directory is within the allowed chroot area,
/// sanitizes inputs to prevent directory traversal attacks, and sends appropriate
/// responses back to the FTP client.
///
/// # Arguments
///
/// * `writer` - A shared, locked TCP stream for writing responses to the client.
/// * `config` - A shared server configuration.
/// * `session` - A shared, locked session containing the user's current state.
/// * `arg` - The current name of the file or directory.
///
/// # Returns
///
/// Result<(), std::io::Error> indicating the success or failure of the operation.
pub async fn handle_rnfr_command(
    writer: Arc<Mutex<TcpStream>>,
    _config: Arc<Config>,
    session: Arc<Mutex<Session>>,
    arg: String,
) -> Result<(), std::io::Error> {
    // Sanitize the input argument to prevent directory traversal attacks.
    let sanitized_arg = sanitize_input(&arg);
    info!("Received RNFR command with argument: {}", sanitized_arg);

    // Construct the path of the file or directory to be renamed.
    let (base_path, path) = {
        // Lock the session to get the current directory.
        let session = session.lock().await;
        let base_path = session.base_path.clone();
        let path = base_path.join(&session.current_dir).join(&sanitized_arg);
        (base_path, path)
    };
    info!("Constructed path: {:?}", path);

    // Canonicalize the path to ensure it's within the chroot directory.
    let resolved_path = path.canonicalize().unwrap_or_else(|_| path.clone());

    // Check if the resolved path is within the chroot directory.
    if !resolved_path.starts_with(&base_path) {
        error!("Path is outside of the allowed area: {:?}", resolved_path);
        send_response(&writer, b"550 Path is outside of the allowed area.\r\n").await?;
        return Ok(());
    }

    // Check if the file or directory exists.
    if !resolved_path.exists() {
        warn!("File or directory does not exist: {:?}", resolved_path);
        send_response(&writer, b"550 File or directory does not exist.\r\n").await?;
        return Ok(());
    }

    // Store the path in the session for use by the RNTO command.
    {
        let mut session = session.lock().await;
        session.rename_from = Some(resolved_path.clone());
    }
    info!("Stored path for renaming: {:?}", resolved_path);

    // Send success response.
    send_response(&writer, b"350 Ready for RNTO.\r\n").await?;
    info!("Sent RNFR success response.");

    Ok(())
}
