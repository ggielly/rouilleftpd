use crate::Config;
use std::sync::Arc;
use tokio::io::AsyncWriteExt;
use tokio::net::TcpStream;
use tokio::sync::Mutex;
use log::{info, error};

/// Handles the USER FTP command.
///
/// This function sets the username for the session and requests the password from the client.
///
/// # Arguments
///
/// * `writer` - A shared, locked TCP stream for writing responses to the client.
/// * `_config` - A shared server configuration (not used in this command).
/// * `_username` - The username provided by the client.
///
/// # Returns
///
/// Result<(), std::io::Error> indicating the success or failure of the operation.
pub async fn handle_user_command(
    writer: Arc<Mutex<TcpStream>>,
    _config: Arc<Config>,
    _username: String,
) -> Result<(), std::io::Error> {
    info!("Received USER command with username: {}", _username);
    
    let mut writer = writer.lock().await;
    if let Err(e) = writer.write_all(b"331 User name okay, need password.\r\n").await {
        error!("Failed to send USER response: {}", e);
        return Err(e);
    }

    info!("Sent response to USER command, awaiting password.");
    Ok(())
}
