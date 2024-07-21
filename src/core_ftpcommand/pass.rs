use crate::core_auth::helper::load_passwd_file;
use crate::session::Session;
use crate::Config;
use bcrypt::verify;
use colored::*;
use log::{error, info, warn};
use std::sync::Arc;
use tokio::io::AsyncWriteExt;
use tokio::net::TcpStream;
use tokio::sync::Mutex;

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
    config: Arc<Config>,
    session: Arc<Mutex<Session>>,
    password: String,
) -> Result<(), std::io::Error> {
    info!("Received PASS command. Authenticating user.");

    let username = {
        let session = session.lock().await;
        session.username.clone()
    };

    let passwd_file_path = config.server.passwd_file.clone();
    let passwd_map = load_passwd_file(&passwd_file_path).await;

    let response: &[u8] = if let Some(username) = username {
        if username.to_lowercase() == "anonymous" {
            info!("Anonymous login with password: {}", password.yellow());
            b"230 Anonymous user logged in, proceed.\r\n"
        } else if let Some(entry) = passwd_map.get(&username) {
            if verify(&password, &entry.get_hashed_password()).unwrap_or(false) {
                {
                    let mut session = session.lock().await;
                    session.is_authenticated = true;
                }
                info!("User {} authenticated successfully.", username.cyan());
                b"230 User logged in, proceed.\r\n"
            } else {
                warn!("Authentication failed for user {}.", username.magenta());
                b"530 Login incorrect.\r\n"
            }
        } else {
            warn!("User {} not found in passwd file.", username.magenta());
            b"530 Login incorrect.\r\n"
        }
    } else {
        warn!(
            "{}",
            "PASS command received without a preceding USER command.".magenta()
        );
        b"503 Bad sequence of commands.\r\n"
    };

    let mut writer = writer.lock().await;
    if let Err(e) = writer.write_all(response).await {
        error!("Failed to send PASS response: {}", e.to_string().red());
        return Err(e);
    }

    info!("Sent PASS response.");
    Ok(())
}
