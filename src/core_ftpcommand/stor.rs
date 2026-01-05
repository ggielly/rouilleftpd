use crate::core_quota::manager::QuotaManager;
use crate::helpers::pad_message;
use crate::{
    helpers::{sanitize_input, send_response},
    session::Session,
    Config,
};
use log::{error, info, warn};
use std::io::ErrorKind;
use std::sync::Arc;
use tokio::{
    fs::File,
    io::{self, AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
    sync::Mutex,
};

use crate::constants::MESSAGE_LENGTH;

/// Handles the STOR (Store File) FTP command.
///
/// This function stores a file uploaded by the client to the server within the user's current directory.
/// The function ensures the file is within the allowed chroot area, sanitizes inputs to prevent directory traversal attacks,
/// and sends appropriate responses back to the FTP client.
///
/// # Arguments
///
/// * `writer` - A shared, locked TCP stream for writing responses to the client.
/// * `config` - A shared server configuration.
/// * `session` - A shared, locked session containing the user's current state.
/// * `arg` - The name of the file to be stored.
/// * `data_stream` - The data connection for receiving the file data.
///
/// # Returns
///
/// Result<(), std::io::Error> indicating the success or failure of the operation.
pub async fn handle_stor_command(
    writer: Arc<Mutex<TcpStream>>,
    config: Arc<Config>,
    session: Arc<Mutex<Session>>,
    arg: String,
    data_stream: Option<Arc<Mutex<TcpStream>>>, // Change data_stream to Option
    quota_manager: Option<Arc<QuotaManager>>,
) -> io::Result<()> {
    if arg.trim().is_empty() {
        warn!("STOR command received with no arguments");
        send_response(&writer, b"501 Syntax error in parameters or arguments.\r\n").await?;
        return Ok(());
    }

    let sanitized_arg = sanitize_input(&arg);
    info!("Received STOR command with argument: {}", sanitized_arg);

    // 0. Check quota before proceeding
    if let Some(quota_mgr) = &quota_manager {
        let session = session.lock().await;
        let username = session
            .username
            .clone()
            .unwrap_or_else(|| "anonymous".to_string());
        let base_dir = session.base_path.clone();

        // Estimate file size (we'll check again after transfer with actual size)
        // For now, we use a reasonable estimate or check if quota allows any upload
        match quota_mgr.check_upload(&username, base_dir, 0).await {
            Ok(_) => {}
            Err(e) => {
                error!("Quota check failed for user {}: {}", username, e);
                send_response(&writer, e.to_ftp_response().as_bytes()).await?;
                return Ok(());
            }
        }
    }

    // 1. Secure Path Construction:
    let file_path = {
        let session = session.lock().await;

        // Ensure the file path is within the base path
        let file_path = session.base_path.join(&sanitized_arg);
        if !file_path.starts_with(&session.base_path) {
            error!("Path is outside of the allowed area: {:?}", file_path);
            send_response(&writer, b"550 Path -is outside of the allowed area.\r\n").await?;
            return Ok(());
        }
        file_path
    };

    // 2. Create File and Handle Errors
    let mut file = match File::create(&file_path).await {
        Ok(f) => f,
        Err(e) => {
            error!("Failed to create file: {:?}, error: {}", file_path, e);
            // More specific error handling based on the type of error
            let message = match e.kind() {
                ErrorKind::NotFound => pad_message(b"550 File not found.\r\n", MESSAGE_LENGTH),
                ErrorKind::PermissionDenied => {
                    pad_message(b"550 Permission denied.\r\n", MESSAGE_LENGTH)
                }
                _ => pad_message(
                    b"451 Requested action aborted. Local error in processing.\r\n",
                    MESSAGE_LENGTH,
                ),
            };

            send_response(&writer, &message).await?; // Pass message as a reference slice

            return Ok(());
        }
    };

    // 3. Data Transfer
    send_response(
        &writer,
        b"150 File status okay; about to open data connection.\r\n",
    )
    .await?;

    // Lock the data stream
    let data_stream = {
        let mut session = session.lock().await;
        session.data_stream.take().expect("Data stream is None")
    };

    let mut data_stream = data_stream.lock().await;
    let buffer_size = config.server.upload_buffer_size.unwrap_or(65536); // Use configured or default buffer size
    let mut buffer = vec![0; buffer_size];

    loop {
        // Read from file into buffer
        let bytes_read = match data_stream.read(&mut buffer).await {
            Ok(0) => break, // End of file
            Ok(n) => n,
            Err(e) => {
                error!("Error reading from data stream: {}", e);
                send_response(&writer, b"550 File read error.\r\n").await?;
                return Ok(());
            }
        };

        // Write from buffer to the file
        if let Err(e) = file.write_all(&buffer[..bytes_read]).await {
            error!("Error writing to file: {}", e);
            return Err(e);
        }
    }

    // Shut down data stream when done
    if let Err(e) = data_stream.shutdown().await {
        error!("Error shutting down data stream: {}", e);
        // Send error response to the client
        send_response(&writer, b"426 Connection closed; transfer aborted.\r\n").await?;
        return Ok(());
    }

    send_response(&writer, b"226 File transfer complete.\r\n").await?;
    info!("File stored successfully: {:?}", file_path);

    // Update quota after successful transfer
    if let Some(quota_mgr) = &quota_manager {
        let session = session.lock().await;
        let username = session
            .username
            .clone()
            .unwrap_or_else(|| "anonymous".to_string());

        // Get the actual file size
        if let Ok(metadata) = tokio::fs::metadata(&file_path).await {
            let file_size = metadata.len();
            if let Err(e) = quota_mgr.record_upload(&username, file_size).await {
                error!("Failed to update quota for user {}: {}", username, e);
            }
        }
    }

    Ok(())
}
