use crate::Config;
use crate::session::Session;
use log::{error, info, warn};
use std::sync::Arc;
use tokio::io::AsyncWriteExt;
use tokio::net::TcpStream;
use tokio::sync::Mutex;

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
    session: Arc<Mutex<Session>>,
    username: String,
) -> Result<(), std::io::Error> {
    info!("Received USER command with username: {}", username);

    {
        let mut session = session.lock().await;
        session.username = Some(username.clone());
    }

    let response: &[u8] = if username.to_lowercase() == "anonymous" {
        info!("Anonymous login initiated for username: {}", username);
        b"331 Anonymous login okay, send your complete email address as password.\r\n"
    } else {
        info!("Username accepted: {}", username);
        b"331 User name okay, need password.\r\n"
    };

    let mut writer = writer.lock().await;
    if let Err(e) = writer.write_all(response).await {
        error!("Failed to send USER response: {}", e);
        return Err(e);
    }

    info!("Sent response to USER command, awaiting password.");
    Ok(())
}
