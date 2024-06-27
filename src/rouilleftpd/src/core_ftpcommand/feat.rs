use anyhow::Result;
use log::{error, info};
use std::sync::Arc;
use tokio::io::AsyncWriteExt;
use tokio::net::TcpStream;
use tokio::sync::Mutex;

/// Handles the FEAT (Feature) FTP command.
///
/// This function responds with a list of supported features.
///
/// # Arguments
///
/// * `writer` - A shared, locked TCP stream for writing responses to the client.
///
/// # Returns
///
/// Result<(), std::io::Error> indicating the success or failure of the operation.
pub async fn handle_feat_command(
    writer: Arc<Mutex<TcpStream>>,
    _arg: String,
) -> Result<(), std::io::Error> {
    // Define the list of supported features
    let features = vec![
       // "UTF8",
      //  "AUTH TLS",
      //  "PBSZ",
      //  "PROT",
       // "MLST type*;size*;modify*;",
       // "REST STREAM",
      //  "SIZE",
      //  "MDTM",
    ];

    // Construct the response
    let mut response = String::from("211-Features:\r\n");
    for feature in features {
        response.push_str(feature);
        response.push_str("\r\n");
    }
    response.push_str("211 End\r\n");

    info!("Responding to FEAT command with supported features.");

    // Lock the writer to send the response.
    let mut writer = writer.lock().await;
    if let Err(e) = writer.write_all(response.as_bytes()).await {
        error!("Failed to send FEAT response: {}", e);
        return Err(e);
    }

    Ok(())
}
