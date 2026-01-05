use crate::tokio::net::TcpStream;
use crate::Config;
use log::{error, info};
use std::io::Error;
use std::sync::Arc;
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
use tokio::io::AsyncWriteExt;
use tokio::sync::Mutex;

pub async fn handle_quit_command(
    writer: Arc<Mutex<TcpStream>>,
    _config: Arc<Config>,
    _arg: String,
) -> Result<(), Error> {
    let mut writer = writer.lock().await;
    info!("Received QUIT command. Closing connection.");

    if let Err(e) = writer
        .write_all(b"221 Service closing control connection.\r\n")
        .await
    {
        error!("Failed to send QUIT response: {}", e);
        return Err(e);
    }

    // Use a formatted string to send the QUIT response
    writer
        .write_all(b"221 Service closing control connection.\r\n")
        .await?;
    info!("Sent QUIT response. Connection closed.");
    Ok(())
}
