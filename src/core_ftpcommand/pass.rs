use crate::Config;
use log::{error, info};
use std::sync::Arc;
use tokio::io::AsyncWriteExt;
use tokio::net::TcpStream;
use tokio::sync::Mutex;

/// Handles the PASS (Password) FTP command.
///
/// This function authenticates the user with the provided password.
/// In this simplified implementation, it always succeeds and logs the user in.
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
    _password: String,
) -> Result<(), std::io::Error> {
    info!("Received PASS command. Authenticating user.");

    let mut writer = writer.lock().await;
    if let Err(e) = writer.write_all(b"230 User logged in, proceed.\r\n").await {
        error!("Failed to send PASS response: {}", e);
        return Err(e);
    }

    info!("User authenticated successfully.");
    Ok(())
}
