use crate::core_auth::helper::load_passwd_file;
use crate::session::Session;
use crate::Config;
use colored::*;
use log::{error, info};
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
    config: Arc<Config>,
    session: Arc<Mutex<Session>>,
    username: String,
) -> Result<(), std::io::Error> {
    info!("Received USER command with username: {}", username);

    let passwd_file_path = config.server.passwd_file.clone();
    let passwd_map = load_passwd_file(&passwd_file_path).await;

    {
        let mut session = session.lock().await;
        session.username = Some(username.clone());
    }

    let response: &[u8] = if username.to_lowercase() == "anonymous" {
        info!("Anonymous login initiated for username: {}", username);
        b"331 Anonymous login okay, send your complete email address as password.\r\n"
    } else if passwd_map.contains_key(&username) {
        info!("Username accepted: {}", username);
        b"331 User name okay, need password.\r\n"
    } else {
        info!("Username not found: {}", username);
        b"530 User not found.\r\n"
    };

    let mut writer = writer.lock().await;
    if let Err(e) = writer.write_all(response).await {
        error!("Failed to send USER response: {}", e);
        return Err(e);
    }

    info!("Sent response to USER command, awaiting password.");
    Ok(())
}
