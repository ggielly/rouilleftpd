// core_ftpcommand/size.rs

use crate::{Config, session::Session};
use log::info;
use log::{error, Metadata};
use std::sync::Arc;
use tokio::{
    io::AsyncWriteExt,
    net::TcpStream,
    sync::Mutex,
    //fs::Metadata,
};

//use crate::core_ftpcommand::helper::{respond_with_error, send_response};

use crate::helpers::send_response;


/// Handles the SIZE (File Size) FTP command.
///
/// This function retrieves the size of a file on the server and sends the size information back to the client.
/// The function ensures the file is within the allowed chroot area, sanitizes inputs to
/// prevent directory traversal attacks, and sends appropriate responses back to the FTP client.
///
/// # Arguments
///
/// * `writer` - A shared, locked TCP stream for writing responses to the client.
/// * `config` - A shared server configuration.
/// * `session` - A shared, locked session containing the user's current state.
/// * `arg` - The name of the file to retrieve its size.
/// * `_data_stream` - Placeholder for the optional data stream (not used in this command).
///
/// # Returns
///
/// Result<(), std::io::Error> indicating the success or failure of the operation.
pub async fn handle_size_command(
    writer: Arc<Mutex<TcpStream>>,
    config: Arc<Config>,
    session: Arc<Mutex<Session>>,
    arg: String,
    _data_stream: Option<Arc<Mutex<TcpStream>>>,
) -> Result<(), std::io::Error> {
    if arg.trim().is_empty() {
        send_response(&writer, b"501 Syntax error in parameters or arguments.\r\n").await?;
        return Ok(());
    }

    let file_path = {
        let session = session.lock().await;
        session.base_path.join(arg.trim_start_matches('/'))
    };
    let base_path = {
        let session = session.lock().await;
        session.base_path.clone()
    };
    let resolved_path = file_path
        .canonicalize()
        .unwrap_or_else(|_| file_path.clone());

    if !resolved_path.starts_with(&base_path) {
        error!("Path is outside of the allowed area: {:?}", resolved_path);
        send_response(&writer, b"550 Requested action not taken (file unavailable or not accessible).\r\n").await?;
        return Ok(());
    }

    // Get file metadata
    let metadata = match tokio::fs::metadata(&resolved_path).await {
        Ok(metadata) => metadata,
        Err(e) => {
            error!("Failed to get file metadata: {:?}, error: {}", resolved_path, e);
            send_response(&writer, b"550 Requested action not taken (file unavailable or not accessible).\r\n").await?;
            return Ok(());
        }
    };

    if !metadata.is_file() {
        send_response(&writer, b"550 Requested action not taken (not a file).\r\n").await?;
        return Ok(());
    }
    
    // Send size response
    let file_size = metadata.len();
    info!("File size for {:?} is {}", resolved_path, file_size);
    send_response(&writer, format!("213 {}\r\n", file_size).as_bytes()).await?;

    Ok(())
}
