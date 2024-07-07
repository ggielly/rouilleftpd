use crate::helpers::{sanitize_input, send_response};
use crate::{session::Session, Config};
use log::{error, info, warn};
use std::io::{self, ErrorKind};
use std::sync::Arc;
use tokio::{
    fs::File,
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
    sync::Mutex,
};

/// Handles the RETR (Retrieve File) FTP command.
pub async fn handle_retr_command(
    writer: Arc<Mutex<TcpStream>>,
    config: Arc<Config>,
    session: Arc<Mutex<Session>>,
    arg: String,
) -> io::Result<()> {
    if arg.trim().is_empty() {
        warn!("RETR command received with no arguments");
        send_response(&writer, b"501 Syntax error in parameters or arguments.\r\n").await?;
        return Ok(());
    }

    let sanitized_arg = sanitize_input(&arg);
    info!("Received RETR command with argument: {}", sanitized_arg);

    let file_path = {
        let session = session.lock().await;
        let file_path = session.base_path.join(&sanitized_arg);

        if !file_path.starts_with(&session.base_path) {
            error!("Path is outside of the allowed area: {:?}", file_path);
            send_response(&writer, b"550 Path is outside of the allowed area.\r\n").await?;
            return Ok(());
        }
        file_path
    };

    let mut file = match File::open(&file_path).await {
        Ok(f) => f,
        Err(e) => {
            error!("Failed to open file: {:?}, error: {}", file_path, e);
            let message = match e.kind() {
                ErrorKind::NotFound => "550 File not found.\r\n",
                ErrorKind::PermissionDenied => "550 Permission denied.\r\n",
                _ => "451 Requested action aborted. Local error in processing.\r\n",
            };
            send_response(&writer, message.as_bytes()).await?;
            return Ok(());
        }
    };

    send_response(&writer, b"150 File status okay; about to open data connection.\r\n").await?;

    let data_stream = {
        let session = session.lock().await;
        session.data_stream.clone()
    };

    if let Some(data_stream) = data_stream {
        let mut data_stream = data_stream.lock().await;
        let buffer_size = config.server.download_buffer_size.unwrap_or(256 * 1024);
        let mut buffer = vec![0; buffer_size]; // Use configured buffer size

        loop {
            info!("Reading from file...");
            let bytes_read = match file.read(&mut buffer).await {
                Ok(0) => {
                    info!("No more data to read from file.");
                    break;
                },
                Ok(n) => n,
                Err(e) => {
                    error!("Error reading from file: {}", e);
                    send_response(&writer, b"550 Error reading from file.\r\n").await?;
                    return Ok(());
                }
            };
            info!("Read {} bytes from file", bytes_read);

            info!("Writing to data stream...");
            if let Err(e) = data_stream.write_all(&buffer[..bytes_read]).await {
                error!("Error writing to data stream: {}", e);
                send_response(&writer, b"550 Error writing to data connection.\r\n").await?;
                return Ok(());
            }
            info!("Wrote {} bytes to data stream", bytes_read);
        }

        // Ensure all data is flushed and the connection is properly closed
        if let Err(e) = data_stream.shutdown().await {
            error!("Error closing data stream: {}", e);
            send_response(&writer, b"426 Connection closed; transfer aborted.\r\n").await?;
            return Ok(());
        }
    } else {
        error!("Data connection is not established.");
        send_response(&writer, b"425 Can't open data connection.\r\n").await?;
        return Ok(());
    }

    send_response(&writer, b"226 Transfer complete.\r\n").await?;
    info!("File transferred successfully: {:?}", file_path);

    Ok(())
}