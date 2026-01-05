use anyhow::Result;
use log::{error, info};
use std::sync::Arc;
use tokio::io::AsyncWriteExt;
use tokio::net::TcpStream;
use tokio::sync::Mutex;

/// Handles the SYST (System) FTP command.
///
/// This function sends a response to the client indicating the system type of the server.
///
/// # Arguments
///
/// * `writer` - A shared, locked TCP stream for writing responses to the client.
///
/// # Returns
///
/// Result<(), std::io::Error> indicating the success or failure of the operation.
pub async fn handle_syst_command(writer: Arc<Mutex<TcpStream>>) -> Result<(), std::io::Error> {
    // Define the system type. Typically "UNIX" for Unix-like systems.
    let system_type = "215 UNIX Type: L8\r\n";

    info!("Responding to SYST command with system type.");

    // Lock the writer to send the response.
    let mut writer = writer.lock().await;
    if let Err(e) = writer.write_all(system_type.as_bytes()).await {
        error!("Failed to send SYST response: {}", e);
        return Err(e);
    }

    Ok(())
}
