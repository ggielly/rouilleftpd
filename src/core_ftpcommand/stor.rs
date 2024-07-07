use crate::{Config, session::Session, helpers::{sanitize_input, send_response}};
use log::{error, info, warn};
use std::sync::Arc;
use tokio::{
    fs::File,
    io::{self, AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
    sync::Mutex,
};
use crate::helpers::pad_message;
use std::io::ErrorKind;

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
    data_stream: Arc<Mutex<TcpStream>>,

) -> io::Result<()> {
    if arg.trim().is_empty() {
        warn!("STOR command received with no arguments");
        send_response(&writer, b"501 Syntax error in parameters or arguments.\r\n").await?;
        return Ok(());
    }

    let sanitized_arg = sanitize_input(&arg);
    info!("Received STOR command with argument: {}", sanitized_arg);

    // 1. Secure Path Construction:
    let file_path = {
        let session = session.lock().await;

        // Ensure the file path is within the base path
        let file_path = session.base_path.join(&sanitized_arg);
        if !file_path.starts_with(&session.base_path) {
            error!("Path is outside of the allowed area: {:?}", file_path);
            send_response(&writer, b"550 Path is outside of the allowed area.\r\n").await?;
            return Ok(());
        }
        file_path
    };
    

    // 2. Create File and Handle Errors:
    let mut file = match File::create(&file_path).await {
        Ok(f) => f,
        Err(e) => {
            error!("Failed to create file: {:?}, error: {}", file_path, e);
            // More specific error handling based on the type of error
            let message = match e.kind() {
                ErrorKind::NotFound => pad_message(b"550 File not found.\r\n", MESSAGE_LENGTH),
                ErrorKind::PermissionDenied => pad_message(b"550 Permission denied.\r\n", MESSAGE_LENGTH),
                _ => pad_message(b"451 Requested action aborted. Local error in processing.\r\n", MESSAGE_LENGTH),
            };
        
            send_response(&writer, &message).await?; // Pass message as a reference slice
        
            return Ok(());
        }
    };

    // 3. Data Transfer
    send_response(&writer, b"150 File status okay; about to open data connection.\r\n").await?;
    
    // Lock the data stream 
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

    Ok(())
}
