use crate::session::Session;
use crate::helpers::{sanitize_input, send_response};
use crate::Config;
use anyhow::Result;
use log::{error, info};
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
    data_stream: Arc<Mutex<TcpStream>>,
    _config: Arc<Config>,
    session: Arc<Mutex<Session>>,
    arg: String,
) -> Result<(), std::io::Error> {
    // Sanitize the input argument to prevent directory traversal attacks.
    let sanitized_arg = sanitize_input(&arg);
    info!("Received STOR command with argument: {}", sanitized_arg);

    // Construct the file path within the user's current directory.
    let file_path = {
        let session = session.lock().await;
        session
            .base_path
            .join(&session.current_dir)
            .join(&sanitized_arg)
    };
    info!("Constructed file path: {:?}", file_path);

    // Canonicalize the file path to ensure it's within the chroot directory.
    let base_path = {
        let session = session.lock().await;
        session.base_path.clone()
    };
    let resolved_path = file_path
        .canonicalize()
        .unwrap_or_else(|_| file_path.clone());

    // Check if the resolved path is within the chroot directory.
    if !resolved_path.starts_with(&base_path) {
        error!("Path is outside of the allowed area: {:?}", resolved_path);
        send_response(&writer, b"550 Path is outside of the allowed area.\r\n").await?;
        return Ok(());
    }

    // Attempt to create the file.
    let mut file = match tokio::fs::File::create(&resolved_path).await {
        Ok(f) => f,
        Err(e) => {
            error!("Failed to create file: {:?}, error: {}", resolved_path, e);
            send_response(&writer, b"550 Failed to create file.\r\n").await?;
            return Ok(());
        }
    };

    // Inform the client that the file status is okay and that the data connection is about to be opened.
    send_response(
        &writer,
        b"150 File status okay; about to open data connection.\r\n",
    )
    .await?;

    // Read data from the data connection and write it to the file.
    let mut data_stream = data_stream.lock().await;
    let mut buffer = vec![0; 8192];

    loop {
        let bytes_read = data_stream.read(&mut buffer).await?;
        if bytes_read == 0 {
            break;
        }
        file.write_all(&buffer[..bytes_read]).await?;
    }

    // Inform the client that the file transfer was successful and the data connection is being closed.
    send_response(
        &writer,
        b"226 Closing data connection. File transfer successful.\r\n",
    )
    .await?;
    info!("File stored successfully: {:?}", resolved_path);

    Ok(())
}
