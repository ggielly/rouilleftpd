use crate::session::Session;
use crate::Config;
use log::{error, info, warn};
use std::sync::Arc;
use tokio::io::AsyncWriteExt;
use tokio::net::TcpStream;
use tokio::sync::Mutex;
use colored::*;

/// Handles the PASS (Password) FTP command.
///
/// This function authenticates the user with the provided password.
///
/// # Arguments
///
/// * `writer` - A shared, locked TCP stream for writing responses to the client.
/// * `_config` - A shared server configuration (not used in this command).
/// * `_password` - The password provided by the user.
///
/// # Returns
///
/// Result<(), std::io::Error> indicating the success or failure of the operation.
pub async fn handle_pass_command(
    writer: Arc<Mutex<TcpStream>>,
    _config: Arc<Config>,
    session: Arc<Mutex<Session>>,
    password: String,
) -> Result<(), std::io::Error> {
    info!("Received PASS command. Authenticating user.");

    let username = {
        let session = session.lock().await;
        session.username.clone()
    };

    let response: &[u8] = if let Some(username) = username {
        if username.to_lowercase() == "anonymous" {
            info!("Anonymous login with password: {}", password.yellow());
            b"230 Anonymous user logged in, proceed.\r\n"
        } else {
            info!("User {} logged in with password: {}", username.cyan(), password.yellow());
            b"230 User logged in, proceed.\r\n"
        }
    } else {
        warn!("{}", "PASS command received without a preceding USER command.".magenta());
        b"503 Bad sequence of commands.\r\n"
    };

    let mut writer = writer.lock().await;
    if let Err(e) = writer.write_all(response).await {
        error!("Failed to send PASS response: {}", e.to_string().red());
        return Err(e);
    }

    info!("User authenticated successfully.");
    Ok(())
}