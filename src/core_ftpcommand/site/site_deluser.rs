use crate::constants::{DELETED, MIN_DELUSER_ARGS, SITE_DELUSER_HELP_PATH};
use crate::core_ftpcommand::site::helper::{respond_with_error, respond_with_success};
use crate::helpers::send_file_to_client;
use crate::{session::Session, Config};
use log::{info, warn};
use std::{
    fs::{self, OpenOptions},
    io::{Read, Write},
    path::PathBuf,
    sync::Arc,
};
use tokio::{net::TcpStream, sync::Mutex};

/// Handles the SITE DELUSER command.
///
/// This command marks a user as deleted by adding a deleted flag to their user file.
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
/// Returns `Ok(())` on success, or an `std::io::Error` on failure

pub async fn handle_site_deluser_command(
    writer: Arc<Mutex<TcpStream>>,
    config: Arc<Config>,
    _session: Arc<Mutex<Session>>,
    args: Vec<String>,
) -> Result<(), std::io::Error> {
    if args.len() < MIN_DELUSER_ARGS {
        warn!("Insufficient arguments for SITE DELUSER: {:?}", args);
        send_file_to_client(&writer, &config.server.chroot_dir, SITE_DELUSER_HELP_PATH).await?;

        respond_with_error(&writer, b"501 Syntax error in parameters or arguments.\r\n").await?;
        return Ok(());
    }

    let username = &args[0];

    let user_file_path = PathBuf::from(&config.server.chroot_dir)
        .join("ftp-data/users")
        .join(format!("{}.user", username));

    if !user_file_path.exists() {
        respond_with_error(&writer, b"550 User not found.\r\n").await?;
        return Ok(());
    }

    let mut user_data = String::new();
    {
        let mut file = fs::File::open(&user_file_path)?;
        file.read_to_string(&mut user_data)?;
    }

    let mut updated_user_data = String::new();
    let mut flag_found = false;

    for line in user_data.lines() {
        let line_trimmed = line.trim();
        if line_trimmed.starts_with("FLAGS ") {
            let flags: Vec<&str> = line_trimmed[6..].split(',').collect();
            if flags.contains(&DELETED.to_string().as_str()) {
                flag_found = true;
                updated_user_data.push_str(line);
                updated_user_data.push('\n');
                continue;
            } else {
                updated_user_data.push_str("FLAGS ");
                updated_user_data.push_str(&flags.join(","));
                updated_user_data.push(',');
                updated_user_data.push_str(&DELETED.to_string());
                updated_user_data.push('\n');
                flag_found = true;
                continue;
            }
        }
        updated_user_data.push_str(line);
        updated_user_data.push('\n');
    }

    if !flag_found {
        updated_user_data.push_str("FLAGS ");
        updated_user_data.push_str(&DELETED.to_string());
        updated_user_data.push('\n');
    }

    let mut file = OpenOptions::new()
        .write(true)
        .truncate(true)
        .open(&user_file_path)?;
    file.write_all(updated_user_data.as_bytes())?;

    info!("User {} marked as deleted.", username);
    respond_with_success(&writer, b"200 User marked as deleted.\r\n").await?;
    Ok(())
}
