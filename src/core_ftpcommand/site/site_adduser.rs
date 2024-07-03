use crate::{session::Session, Config};
use log::{error, info, warn};
use std::{
    fs::{self, OpenOptions},
    io::Write,
    path::{Path, PathBuf},
    sync::Arc,
};
use tokio::{net::TcpStream, sync::Mutex};

use crate::core_ftpcommand::site::helper::{
    is_valid_ip_or_hostname, 
    is_valid_password, 
    is_valid_username, 
    respond_with_error,
    respond_with_success,
};

/// Handles the SITE ADDUSER command.
///
/// This command adds a new user to the FTP server.
///
/// # Arguments
///
/// * `writer` - The TCP stream writer to send responses.
/// * `config` - The server configuration.
/// * `session` - The current FTP session (not used in this command).
/// * `args` - The command arguments.
///
/// # Returns
///
/// Returns `Ok(())` on success, or an `std::io::Error` on failure.
pub async fn handle_site_adduser_command(
    writer: Arc<Mutex<TcpStream>>,
    config: Arc<Config>,
    _session: Arc<Mutex<Session>>, // Session not used in this command
    args: Vec<String>,
) -> Result<(), std::io::Error> {
    const MIN_ARGS: usize = 2;

    if args.len() < MIN_ARGS {
        warn!("Insufficient arguments for SITE ADDUSER: {:?}", args);
        respond_with_error(&writer, b"501 Syntax error in parameters or arguments.\r\n").await?;
        return Ok(());
    }

    let username = &args[0];
    let password = &args[1];
    let idents_ips = &args[2..];

    // Validate username and password
    if !is_valid_username(username) || !is_valid_password(password) {
        respond_with_error(&writer, b"501 Invalid username or password.\r\n").await?;
        return Ok(());
    }

    let user_file_path = PathBuf::from(&config.server.chroot_dir)
        .join("ftp-data/users")
        .join(format!("{}.user", username));

    // Check if user already exists
    if user_file_path.exists() {
        respond_with_error(&writer, b"550 User already exists.\r\n").await?;
        return Ok(());
    }

    // Create user file based on default template
    match create_user_file(&user_file_path, username, password, idents_ips) {
        Ok(_) => {
            info!("User {} added successfully", username);
            respond_with_success(&writer, b"200 User added successfully.\r\n").await?;
        }
        Err(e) => {
            error!("Failed to create user file: {}", e);
            respond_with_error(&writer, b"550 Failed to create user.\r\n").await?;
        }
    }
    Ok(())
}

/// Creates a new user file based on a default template.
///
/// # Arguments
///
/// * `user_file_path` - The path to the user file.
/// * `username` - The username for the new user.
/// * `password` - The password for the new user.
/// * `ips` - A slice of allowed IP addresses or hostnames.
///
/// # Returns
///
/// Returns `Ok(())` on success, or an `std::io::Error` on failure.
fn create_user_file(
    user_file_path: &Path,
    username: &str,
    password: &str,
    ips: &[String],
) -> std::io::Result<()> {
    let default_user_file = PathBuf::from("ftp-data/users/default.user");
    let mut user_data = fs::read_to_string(&default_user_file)?;

    // Customize user data
    user_data = user_data.replace("No Tagline Set", username);
    user_data.push_str(&format!("USER {}\n", username));
    user_data.push_str(&format!("PASSWORD {}\n", password));

    for ip in ips {
        if !is_valid_ip_or_hostname(ip) {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                format!("Invalid IP or hostname: {}", ip),
            ));
        }
        user_data.push_str(&format!("IP {}\n", ip));
    }

    // Write to new user file
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(user_file_path)?;
    file.write_all(user_data.as_bytes())?;

    Ok(())
}
