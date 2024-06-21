use std::sync::Arc;
use tokio::io::AsyncWriteExt;
use tokio::net::TcpStream;
use tokio::sync::Mutex;
use crate::Config;
use log::{info, error};

/// Handles the NOOP (No Operation) FTP command.
///
/// This function responds with a 200 OK message to indicate the server is alive.
///
/// # Arguments
///
/// * `writer` - A shared, locked TCP stream for writing responses to the client.
/// * `_config` - A shared server configuration (not used in this command).
/// * `_arg` - The argument for the NOOP command (not used in this command).
///
/// # Returns
///
/// Result<(), std::io::Error> indicating the success or failure of the operation.
pub async fn handle_noop_command(
    writer: Arc<Mutex<TcpStream>>,
    _config: Arc<Config>,
    _arg: String,
) -> Result<(), std::io::Error> {
    let mut writer = writer.lock().await;
    info!("Received NOOP command. Sending OK response.");
    
    if let Err(e) = writer.write_all(b"200 OK, n00p n00p !\r\n").await {
        error!("Failed to send NOOP response: {}", e);
        return Err(e);
    }

    info!("Sent NOOP response successfully.");
    Ok(())
}
