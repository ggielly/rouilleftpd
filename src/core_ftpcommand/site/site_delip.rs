use crate::constants::{MIN_DELIP_ARGS, MAX_DELIP_IPS};
use crate::{session::Session, Config};
use log::{error, info, warn};
use std::{
    fs::{self, OpenOptions},
    io::{Read, Write},
    path::{Path, PathBuf},
    sync::Arc,
};
use tokio::{io::AsyncWriteExt, net::TcpStream, sync::Mutex};

use crate::core_ftpcommand::site::helper::{
    is_valid_ident_ip, is_valid_ip_or_hostname, respond_with_error, respond_with_success,
};

pub async fn handle_delip_command(
    writer: Arc<Mutex<TcpStream>>,
    config: Arc<Config>,
    _session: Arc<Mutex<Session>>,
    args: Vec<String>,
) -> Result<(), std::io::Error> {
    if args.len() < MIN_DELIP_ARGS {
        warn!("Insufficient arguments for SITE DELIP: {:?}", args);
        respond_with_error(&writer, b"501 Syntax error in parameters or arguments.\r\n").await?;
        return Ok(());
    }

    let username = &args[0];
    let del_ips = &args[1..];

    if del_ips.len() > MAX_DELIP_IPS {
        warn!("Too many IPs for SITE DELIP: {:?}", args);
        respond_with_error(&writer, b"501 Too many IPs. Max 10 allowed.\r\n").await?;
        return Ok(());
    }

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
    let mut ips_removed = 0;

    for line in user_data.lines() {
        let line_trimmed = line.trim();
        if line_trimmed.starts_with("IP ") {
            let ip = &line_trimmed[3..];
            if del_ips.contains(&ip.to_string()) || del_ips.contains(&line_trimmed[3..].to_string())
            {
                ips_removed += 1;
                continue;
            }
        }
        updated_user_data.push_str(line);
        updated_user_data.push('\n');
    }

    if ips_removed == 0 {
        respond_with_error(&writer, b"550 No matching IPs found.\r\n").await?;
        return Ok(());
    }

    let mut file = OpenOptions::new()
        .write(true)
        .truncate(true)
        .open(&user_file_path)?;
    file.write_all(updated_user_data.as_bytes())?;

    info!("IPs removed for user {}: {:?}", username, del_ips);
    respond_with_success(&writer, b"200 IP(s) removed successfully.\r\n").await?;
    Ok(())
}
