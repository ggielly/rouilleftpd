use crate::helpers::{sanitize_input, send_response};
use crate::session::Session;
use crate::Config;
use anyhow::Result;
use log::{error, info, warn};
use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio::sync::Mutex;

/// Handles the STOR (Store File) FTP command.
///
/// This function stores a file uploaded by the client to the server within the user's current directory.
/// The function ensures the file is within the allowed chroot area, sanitizes inputs to prevent directory traversal attacks,
/// and sends appropriate responses back to the FTP client.
///
/// # Arguments
///
/// * `writer` - A shared, locked TCP stream for writing responses to the client.
/// * `data_stream` - The data connection for receiving the file data.
/// * `config` - A shared server configuration.
/// * `session` - A shared, locked session containing the user's current state.
/// * `arg` - The name of the file to be stored.
///
/// # Returns
///
/// Result<(), std::io::Error> indicating the success or failure of the operation.
pub async fn handle_stor_command(
    writer: Arc<Mutex<TcpStream>>,
    _config: Arc<Config>,
    session: Arc<Mutex<Session>>,
    arg: String,
    _data_stream: Option<Arc<Mutex<TcpStream>>>,
) -> Result<(), std::io::Error> {
    if arg.trim().is_empty() {
        warn!("STOR command received with no arguments");
        send_response(&writer, b"501 Syntax error in parameters or arguments.\r\n").await?;
        return Ok(());
    }

    let sanitized_arg = sanitize_input(&arg);
    info!("Received STOR command with argument: {}", sanitized_arg);

    // Corrected file_path construction
    let (base_path, file_path, resolved_path) = {
        let session = session.lock().await;
        let base_path = session.base_path.clone();
        let file_path = base_path.join(&sanitized_arg);
        let resolved_path = file_path
            .canonicalize()
            .unwrap_or_else(|_| file_path.clone());

        (base_path, file_path, resolved_path)
    };

    info!("base_path: {:?}", base_path);
    info!("file_path: {:?}", file_path);
    info!("resolved_path: {:?}", resolved_path);

    if !resolved_path.starts_with(&base_path) {
        error!("Path is outside of the allowed area: {:?}", resolved_path);
        send_response(&writer, b"550 Path is outside of the allowed area.\r\n").await?;
        return Ok(());
    }

    let mut file = match tokio::fs::File::create(&resolved_path).await {
        Ok(f) => f,
        Err(e) => {
            error!("Failed to create file: {:?}, error: {}", resolved_path, e);
            send_response(&writer, b"550 Failed to create file.\r\n").await?;
            return Ok(());
        }
    };

    send_response(
        &writer,
        b"150 File status okay; about to open data connection.\r\n",
    )
    .await?;

    let data_stream = {
        let session = session.lock().await;
        session.data_stream.clone() // Clone the data stream to use it
    };

    if let Some(data_stream) = data_stream {
        info!("Attempting to lock data stream...");
        let mut data_stream = data_stream.lock().await;
        info!("Data stream locked successfully");

        let mut buffer = vec![0; 8192];
        loop {
            info!("Reading from data stream...");
            let bytes_read = match data_stream.read(&mut buffer).await {
                Ok(0) => break,
                Ok(n) => n,
                Err(e) => {
                    error!("Error reading from data stream: {}", e);
                    send_response(&writer, b"550 Error reading from data connection.\r\n").await?;
                    return Ok(());
                }
            };
            info!("Read {} bytes from data stream", bytes_read);

            info!("Writing to file...");
            if let Err(e) = file.write_all(&buffer[..bytes_read]).await {
                error!("Error writing to file: {}", e);
                send_response(&writer, b"550 Error writing to file.\r\n").await?;
                return Ok(());
            }
            info!("Wrote {} bytes to file", bytes_read);
        }
    } else {
        error!("Data connection is not established.");
        send_response(&writer, b"425 Can't open data connection.\r\n").await?;
        return Ok(());
    }

    send_response(&writer, b"226 Transfer complete.\r\n").await?;
    info!("File stored successfully: {:?}", resolved_path);

    Ok(())
}
