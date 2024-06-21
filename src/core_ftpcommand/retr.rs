use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio::sync::Mutex;
use crate::Config;
use std::path::PathBuf;
use tokio::fs::File;
use crate::core_network::Session;
use crate::core_ftpcommand::utils::{sanitize_input, construct_path, send_response};
use anyhow::Result;

/// Handles the RETR (Retrieve) FTP command.
///
/// This function retrieves a file from the server and sends its contents to the client.
/// The function ensures the file is within the allowed chroot area, sanitizes inputs to
/// prevent directory traversal attacks, and sends appropriate responses back to the FTP client.
///
/// # Arguments
///
/// * `writer` - A shared, locked TCP stream for writing responses to the client.
/// * `config` - A shared server configuration.
/// * `session` - A shared, locked session containing the user's current state.
/// * `arg` - The name of the file to retrieve.
///
/// # Returns
///
/// Result<(), std::io::Error> indicating the success or failure of the operation.
pub async fn handle_retr_command(
    writer: Arc<Mutex<TcpStream>>,
    config: Arc<Config>,
    session: Arc<Mutex<Session>>,
    arg: String,
) -> Result<(), std::io::Error> {
    // Sanitize the input argument to prevent directory traversal attacks.
    let sanitized_arg = sanitize_input(&arg);

    // Construct the path of the file to retrieve.
    let file_path = {
        // Lock the session to get the current directory.
        let session = session.lock().await;
        construct_path(&config, &session.current_dir, &sanitized_arg)
    };

    // Canonicalize the chroot directory to resolve any symbolic links or relative paths.
    let chroot_dir = PathBuf::from(&config.server.chroot_dir).canonicalize()?;
    // Canonicalize the file path to ensure it's within the chroot directory.
    let resolved_path = file_path.canonicalize().unwrap_or_else(|_| file_path.clone());

    // Check if the resolved path is within the chroot directory.
    if !resolved_path.starts_with(&chroot_dir) {
        send_response(&writer, b"550 Path is outside of the allowed area.\r\n").await?;
        return Ok(());
    }

    // Attempt to open the file.
    let mut file = match File::open(&resolved_path).await {
        Ok(f) => f,
        Err(_) => {
            send_response(&writer, b"550 File not found.\r\n").await?;
            return Ok(());
        }
    };

    // Send success response indicating that the file transfer is starting.
    send_response(&writer, b"150 Opening data connection.\r\n").await?;

    // Read the file contents and send them to the client.
    let mut buffer = vec![0; 8192];
    loop {
        let bytes_read = file.read(&mut buffer).await?;
        if bytes_read == 0 {
            break;
        }
        writer.lock().await.write_all(&buffer[..bytes_read]).await?;
    }

    // Send transfer complete response.
    send_response(&writer, b"226 Transfer complete.\r\n").await?;

    Ok(())
}
