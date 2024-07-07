use crate::constants::MESSAGE_LENGTH;
use crate::helpers::pad_message;
use crate::{
    helpers::{sanitize_input, send_response},
    session::Session,
    Config,
};
use log::{debug, error, info, trace, warn};
use std::io::ErrorKind;
use std::sync::Arc;
use tokio::{
    fs::File,
    io::{self, AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
    sync::Mutex,
};

/// Handles the RETR (Retrieve File) FTP command.
///
/// This function retrieves a file from the server and sends it to the client over the specified data connection.
/// It ensures the file is within the allowed chroot area, sanitizes inputs, and handles errors gracefully.
pub async fn handle_retr_command(
    writer: Arc<Mutex<TcpStream>>,
    config: Arc<Config>,
    session: Arc<Mutex<Session>>,
    arg: String,
    data_stream: Option<Arc<Mutex<TcpStream>>>,
) -> io::Result<()> {
    if arg.trim().is_empty() {
        warn!("RETR command received with no arguments");
        send_response(&writer, b"501 Syntax error in parameters or arguments.\r\n").await?;
        return Ok(());
    }

    let sanitized_arg = sanitize_input(&arg);
    info!("Received RETR command with argument: {}", sanitized_arg);

    // 1. Secure Path Construction:
    let file_path = {
        let session = session.lock().await;
        let file_path = session.base_path.join(&sanitized_arg);

        // Ensure the file path is within the base path
        if !file_path.starts_with(&session.base_path) {
            error!("Path is outside of the allowed area: {:?}", file_path);
            send_response(&writer, b"550 Path is outside of the allowed area.\r\n").await?;
            return Ok(());
        }
        file_path
    };

    // 2. Open File and Handle Errors:
    let mut file = match File::open(&file_path).await {
        Ok(file) => file,
        Err(err) => {
            error!("Failed to open file: {:?}, error: {}", file_path, err);
            let message = match err.kind() {
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

    // 3. Data Transfer:
    info!("Sending file: {:?}", file_path);
    send_response(
        &writer,
        b"150 Opening data connection for file transfer.\r\n",
    )
    .await?;

    // Ensure data_stream is provided
    trace!("Attempting to lock data stream for file transfer.");
    let data_stream = match data_stream {
        Some(stream) => {
            trace!("Data stream found, locking...");
            stream
        }
        None => {
            error!("Data stream is None");
            send_response(&writer, b"425 Can't open data connection.\r\n").await?;
            return Ok(());
        }
    };

    let mut data_stream = data_stream.lock().await;
    trace!("Data stream locked for file transfer.");
    let buffer_size = config.server.upload_buffer_size.unwrap_or(65536);
    let mut buffer = vec![0; buffer_size];

    loop {
        // Read from file into buffer
        let bytes_read = match file.read(&mut buffer).await {
            Ok(0) => break, // End of file
            Ok(n) => n,
            Err(e) => {
                error!("Error reading file: {}", e);
                send_response(&writer, b"550 File read error.\r\n").await?;
                return Ok(());
            }
        };

        // Write from buffer to the data stream
        if let Err(e) = data_stream.write_all(&buffer[..bytes_read]).await {
            error!("Error writing to data stream: {}", e);
            return Err(e);
        }
        trace!("Transferred {} bytes to data stream.", bytes_read);
    }

    // Shut down data stream when done
    if let Err(e) = data_stream.shutdown().await {
        error!("Error shutting down data stream: {}", e);
        // Send error response to the client
        send_response(&writer, b"426 Connection closed; transfer aborted.\r\n").await?;
        return Ok(());
    }

    send_response(&writer, b"226 File transfer complete.\r\n").await?;
    info!("File transferred successfully: {:?}", file_path);

    Ok(())
}
