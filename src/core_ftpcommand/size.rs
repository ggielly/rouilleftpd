// core_ftpcommand/size.rs

use crate::{session::Session, Config};

use std::sync::Arc;
use tokio::{net::TcpStream, sync::Mutex};

use crate::helpers::sanitize_input;
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
    _config: Arc<Config>,
    session: Arc<Mutex<Session>>,
    arg: String,
    _data_stream: Option<Arc<Mutex<TcpStream>>>,
) -> Result<(), std::io::Error> {
    if arg.trim().is_empty() {
        send_response(&writer, b"501 Syntax error in parameters or arguments.\r\n").await?;
        return Ok(());
    }

    let sanitized_arg = sanitize_input(&arg);
    let (base_path, file_path) = {
        let session = session.lock().await;
        let base_path = session.base_path.clone();
        let file_path = base_path.join(&sanitized_arg);
        (base_path, file_path)
    };

    if !file_path.exists() {
        send_response(&writer, b"550 File not found.\r\n").await?;
        return Ok(());
    }

    let metadata = tokio::fs::metadata(&file_path).await?;
    let file_size = metadata.len();

    let response = format!("213 {}\r\n", file_size);
    send_response(&writer, response.as_bytes()).await?;
    Ok(())
}
