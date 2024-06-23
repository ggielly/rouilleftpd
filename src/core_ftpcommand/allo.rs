use anyhow::Result;
use log::{error, info};
use std::sync::Arc;
use tokio::io::AsyncWriteExt;
use tokio::net::TcpStream;
use tokio::sync::Mutex;

/// Handles the ALLO (Allocate) FTP command.
///
/// This function acknowledges the storage allocation request from the client.
/// Modern systems typically do not need to pre-allocate space, so this implementation
/// simply responds with a success message.
///
/// # Arguments
///
/// * `writer` - A shared, locked TCP stream for writing responses to the client.
/// * `arg` - The number of bytes to allocate (ignored in this implementation).
///
/// # Returns
///
/// Result<(), std::io::Error> indicating the success or failure of the operation.
pub async fn handle_allo_command(
    writer: Arc<Mutex<TcpStream>>,
    _arg: String,
) -> Result<(), std::io::Error> {
    // Log the received ALLO command.
    info!("Received ALLO command with argument: {}", _arg);

    // Define the success response message.
    let response = "200 ALLO command successful.\r\n";

    // Lock the writer to send the response.
    let mut writer = writer.lock().await;
    if let Err(e) = writer.write_all(response.as_bytes()).await {
        error!("Failed to send ALLO response: {}", e);
        return Err(e);
    }

    info!("Sent ALLO success response.");
    Ok(())
}
