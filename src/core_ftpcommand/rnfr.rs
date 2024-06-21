use crate::core_ftpcommand::utils::{construct_path, sanitize_input, send_response};
use crate::core_network::Session;
use crate::Config;
use anyhow::Result;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::io::AsyncWriteExt;
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
    config: Arc<Config>,
    session: Arc<Mutex<Session>>,
    arg: String,
) -> Result<(), std::io::Error> {
    // Sanitize the input argument to prevent directory traversal attacks.
    let sanitized_arg = sanitize_input(&arg);

    // Construct the path of the file or directory to be renamed.
    let path = {
        // Lock the session to get the current directory.
        let session = session.lock().await;
        construct_path(&config, &session.current_dir, &sanitized_arg)
    };

    // Canonicalize the chroot directory to resolve any symbolic links or relative paths.
    let chroot_dir = PathBuf::from(&config.server.chroot_dir).canonicalize()?;
    // Canonicalize the path to ensure it's within the chroot directory.
    let resolved_path = path.canonicalize().unwrap_or_else(|_| path.clone());

    // Check if the resolved path is within the chroot directory.
    if !resolved_path.starts_with(&chroot_dir) {
        send_response(&writer, b"550 Path is outside of the allowed area.\r\n").await?;
        return Ok(());
    }

    // Check if the file or directory exists.
    if !resolved_path.exists() {
        send_response(&writer, b"550 File or directory does not exist.\r\n").await?;
        return Ok(());
    }

    // Store the path in the session for use by the RNTO command.
    {
        let mut session = session.lock().await;
        session.rename_from = Some(resolved_path);
    }

    // Send success response.
    send_response(&writer, b"350 Ready for RNTO.\r\n").await?;

    Ok(())
}
