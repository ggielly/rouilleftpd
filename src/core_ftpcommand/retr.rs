use crate::core_network::Session;
use crate::helpers::{sanitize_input, send_response};
use crate::Config;
use anyhow::Result;
use log::{error, info, warn};
use std::path::PathBuf;
use std::sync::Arc;
use tokio::fs::File;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio::sync::Mutex;

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
    _config: Arc<Config>,
    session: Arc<Mutex<Session>>,
    arg: String,
) -> Result<(), std::io::Error> {
    if arg.trim().is_empty() {
        warn!("RETR command received with no arguments");
        send_response(&writer, b"501 Syntax error in parameters or arguments.\r\n").await?;
        return Ok(());
    }

    let sanitized_arg = sanitize_input(&arg);
    let file_path = {
        let session = session.lock().await;
        session
            .base_path
            .join(&session.current_dir)
            .join(&sanitized_arg)
    };

    let (base_path, resolved_path) = {
        let session = session.lock().await;
        let base_path = session.base_path.clone();
        let resolved_path = file_path
            .canonicalize()
            .unwrap_or_else(|_| file_path.clone());
        (base_path, resolved_path)
    };

    if !resolved_path.starts_with(&base_path) {
        error!("Path is outside of the allowed area: {:?}", resolved_path);
        send_response(&writer, b"550 Path is outside of the allowed area.\r\n").await?;
        return Ok(());
    }

    let mut file = match File::open(&resolved_path).await {
        Ok(f) => f,
        Err(e) => {
            error!(
                "File not found or could not be opened: {:?}, error: {}",
                resolved_path, e
            );
            send_response(&writer, b"550 File not found.\r\n").await?;
            return Ok(());
        }
    };

    send_response(&writer, b"150 Opening data connection.\r\n").await?;
    info!("Sending file: {:?}", resolved_path);

    let mut buffer = vec![0; 8192];
    loop {
        let bytes_read = match file.read(&mut buffer).await {
            Ok(0) => break,
            Ok(n) => n,
            Err(e) => {
                error!("Error reading file: {}", e);
                send_response(&writer, b"550 Error reading file.\r\n").await?;
                return Ok(());
            }
        };
        if let Err(e) = writer.lock().await.write_all(&buffer[..bytes_read]).await {
            error!("Error sending file to client: {}", e);
            return Ok(());
        }
    }

    send_response(&writer, b"226 Transfer complete.\r\n").await?;
    info!("File transfer completed successfully: {:?}", resolved_path);

    Ok(())
}
