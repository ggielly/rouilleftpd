use crate::constants::{MAX_ADDIP_IPS, MIN_ADDIP_ARGS, SITE_ADDIP_HELP_PATH};
use crate::core_ftpcommand::site::helper::{
    is_valid_ident_ip, respond_with_error, respond_with_success,
};
use crate::helpers::send_file_to_client;
use crate::{session::Session, Config};
use log::{info, warn};
use std::{
    fs::{self, OpenOptions},
    io::Write,
    path::{Path, PathBuf},
    sync::Arc,
};
use tokio::{net::TcpStream, sync::Mutex};

pub async fn handle_site_addip_command(
    writer: Arc<Mutex<TcpStream>>,
    config: Arc<Config>,
    _session: Arc<Mutex<Session>>, // Session not used in this command
    args: Vec<String>,
) -> Result<(), std::io::Error> {
    if args.len() < MIN_ADDIP_ARGS {
        warn!("Insufficient arguments for SITE ADDIP: {:?}", args);
        send_file_to_client(&writer, &config.server.chroot_dir, SITE_ADDIP_HELP_PATH).await?;

        respond_with_error(&writer, b"501 Syntax error in parameters or arguments.\r\n").await?;

        return Ok(());
    }

    let username = &args[0];
    let idents_ips = &args[1..];

    if idents_ips.len() > MAX_ADDIP_IPS {
        warn!("Too many IPs for SITE ADDIP: {:?}", idents_ips);
        respond_with_error(&writer, b"501 Too many IPs, max 10 allowed.\r\n").await?;
        // Display help
        return Ok(());
    }

    // Validate and collect idents and IPs
    let mut valid_idents_ips = Vec::new();
    for ident_ip in idents_ips {
        if !is_valid_ident_ip(ident_ip) {
            respond_with_error(
                &writer,
                format!("501 Invalid ident@IP or ident@hostname: {}\r\n", ident_ip).as_bytes(),
            )
            .await?;
            return Ok(());
        }
        valid_idents_ips.push(ident_ip.clone());
    }

    let user_file_path = PathBuf::from(&config.server.chroot_dir)
        .join("ftp-data/users")
        .join(format!("{}.user", username));

    // Check if user exists
    if !user_file_path.exists() {
        respond_with_error(&writer, b"550 User does not exist.\r\n").await?;
        return Ok(());
    }

    // Add IPs to user file
    match add_idents_ips_to_user_file(&user_file_path, valid_idents_ips) {
        Ok(_) => {
            info!("Ident@IPs added to user {} successfully", username);
            respond_with_success(&writer, b"200 ident@IPs added successfully.\r\n").await?;
        }
        Err(e) => {
            warn!("Failed to add ident@IPs to user file: {}", e);
            respond_with_error(&writer, b"550 Failed to add ident@IPs.\r\n").await?;
        }
    }
    Ok(())
}

fn add_idents_ips_to_user_file(
    user_file_path: &Path,
    idents_ips: Vec<String>,
) -> std::io::Result<()> {
    let mut user_data = fs::read_to_string(user_file_path)?;

    for ident_ip in idents_ips {
        user_data.push_str(&format!("IP {}\n", ident_ip));
    }

    let mut file = OpenOptions::new()
        .write(true)
        .truncate(true)
        .open(user_file_path)?;
    file.write_all(user_data.as_bytes())?;

    Ok(())
}
