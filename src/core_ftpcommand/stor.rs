use crate::{
    helpers::{sanitize_input, send_response},
    session::Session,
    Config,
};
use log::{error, info, warn};
use std::sync::Arc;
use tokio::{
    fs::File,
    io::{self, AsyncReadExt, AsyncWriteExt, Result},
    net::TcpStream,
    sync::Mutex,
};

use std::io::ErrorKind;

/// Handles the STOR (Store File) FTP command.

/// Handles the STOR (Store File) FTP command.
pub async fn handle_stor_command(
    writer: Arc<Mutex<TcpStream>>,
    config: Arc<Config>, // Use the config for buffer size
    session: Arc<Mutex<Session>>,
    arg: String,
    _data_stream: Option<Arc<Mutex<TcpStream>>>,
) -> io::Result<()> {
    if arg.trim().is_empty() {
        warn!("STOR command received with no arguments");
        send_response(&writer, b"501 Syntax error in parameters or arguments.\r\n").await?;
        return Ok(());
    }

    let sanitized_arg = sanitize_input(&arg);
    info!("Received STOR command with argument: {}", sanitized_arg);

    // Secure Path Construction:
    let (base_path, file_path) = {
        let mut session = session.lock().await;

        // Get the relative path within the chroot directory
        let relative_current_dir = session
            .current_dir
            .strip_prefix("/")
            .unwrap_or(&session.current_dir);
        let file_path = session
            .base_path
            .join(relative_current_dir)
            .join(&sanitized_arg);

        // Ensure the file path is still within the base path after adjustment
        if !file_path.starts_with(&session.base_path) {
            error!("Path is outside of the allowed area: {:?}", file_path);
            send_response(&writer, b"550 Path is outside of the allowed area.\r\n").await?;
            return Ok(());
        }
        (session.base_path.clone(), file_path)
    };

    // Create File and Handle Errors:
    let mut file = match File::create(&file_path).await {
        Ok(f) => f,
        Err(e) => {
            error!("Failed to create file: {:?}, error: {}", file_path, e);
            // More specific error handling based on the type of error
            let message = match e.kind() {
                ErrorKind::NotFound => "550 File not found.\r\n",
                ErrorKind::PermissionDenied => "550 Permission denied.\r\n",
                _ => "451 Requested action aborted. Local error in processing.\r\n",
            };
            send_response(&writer, message.as_bytes()).await?;
            return Ok(());
        }
    };

    // Data Transfer
    send_response(
        &writer,
        b"150 File status okay; about to open data connection.\r\n",
    )
    .await?;
    let data_stream = {
        let session = session.lock().await;
        session.data_stream.clone()
    };

    if let Some(data_stream) = data_stream {
        info!("Attempting to lock data stream...");
        let mut data_stream = data_stream.lock().await;
        info!("Data stream locked successfully");

        let buffer_size = config.server.upload_buffer_size.unwrap_or(256 * 1024);
        let mut buffer = vec![0; buffer_size]; // Use configured buffer size

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
    info!("File stored successfully: {:?}", file_path);

    Ok(())
}
