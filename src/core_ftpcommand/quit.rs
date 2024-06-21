use tokio::io::{AsyncWriteExt, Result};
use tokio::net::TcpStream;
use tokio::sync::Mutex;
use std::sync::Arc;
use crate::Config;
use log::{info, error};

/// Handles the QUIT FTP command.
///
/// This function sends a response indicating the service is closing the control connection.
///
/// # Arguments
///
/// * `writer` - A shared, locked TCP stream for writing responses to the client.
/// * `_config` - A shared server configuration (not used in this command).
/// * `_arg` - The argument for the QUIT command (not used in this command).
///
/// # Returns
///
/// Result<(), std::io::Error> indicating the success or failure of the operation.
pub async fn handle_quit_command(writer: Arc<Mutex<TcpStream>>, _config: Arc<Config>, _arg: String) -> Result<()> {
    let mut writer = writer.lock().await;
    info!("Received QUIT command. Closing connection.");
    
    if let Err(e) = writer.write_all(b"221 Service closing control connection.\r\n").await {
        error!("Failed to send QUIT response: {}", e);
        return Err(e);
    }

    info!("Sent QUIT response. Connection closed.");
    Ok(())
}
