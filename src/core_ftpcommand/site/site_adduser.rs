use crate::Config;
use crate::session::Session;
use log::{info, warn};
use std::sync::Arc;
use tokio::io::AsyncWriteExt;
use tokio::net::TcpStream;
use tokio::sync::Mutex;
use std::fs::OpenOptions;
use std::io::Write;
use std::path::Path;

pub async fn handle_adduser_command(
    writer: Arc<Mutex<TcpStream>>,
    config: Arc<Config>,
    session: Arc<Mutex<Session>>,
    args: Vec<String>,
) -> Result<(), std::io::Error> {
    if args.len() < 2 {
        warn!("Insufficient arguments for site adduser command: {:?}", args);
        let mut writer = writer.lock().await;
        writer.write_all(b"501 Syntax error in parameters or arguments.\r\n").await?;
        return Ok(());
    }

    let username = &args[0];
    let password = &args[1];
    let idents_ips = &args[2..];

    // Perform the necessary validations
    if !is_valid_username(username) {
        let mut writer = writer.lock().await;
        writer.write_all(b"501 Invalid username.\r\n").await?;
        return Ok(());
    }

    // Create user file path
    let user_file_path = format!("{}/ftp-data/users/{}.user", config.server.chroot_dir, username);

    // Check if the user already exists
    if Path::new(&user_file_path).exists() {
        let mut writer = writer.lock().await;
        writer.write_all(b"550 User already exists.\r\n").await?;
        return Ok(());
    }

    // Create the user file with default values
    let mut user_file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&user_file_path)?;

    writeln!(user_file, "USER {}", username)?;
    writeln!(user_file, "GENERAL 0,0 -1 0 0")?;
    writeln!(user_file, "FLAGS 1")?;
    writeln!(user_file, "LOGINS 2 0 -1 -1")?;
    writeln!(user_file, "TIMEFRAME 0 0")?;
    writeln!(user_file, "TAGLINE No Tagline Set")?;
    writeln!(user_file, "DIR /")?;
    writeln!(user_file, "ADDED 0")?;
    writeln!(user_file, "EXPIRES 0")?;
    writeln!(user_file, "ADDEDBY System")?;
    writeln!(user_file, "CREDITS 15000")?;
    writeln!(user_file, "RATIO 3")?;
    writeln!(user_file, "ALLUP 0 0 0")?;
    writeln!(user_file, "ALLDN 0 0 0")?;
    writeln!(user_file, "WKUP 0 0 0")?;
    writeln!(user_file, "WKDN 0 0 0")?;
    writeln!(user_file, "DAYUP 0 0 0")?;
    writeln!(user_file, "DAYDN 0 0 0")?;
    writeln!(user_file, "MONTHUP 0 0 0")?;
    writeln!(user_file, "MONTHDN 0 0 0")?;
    writeln!(user_file, "NUKE 0 0 0")?;
    writeln!(user_file, "TIME 0 0 0 0")?;
    writeln!(user_file, "IP *")?;

    for ident_ip in idents_ips {
        writeln!(user_file, "IP {}", ident_ip)?;
    }

    // Send success response
    let mut writer = writer.lock().await;
    writer.write_all(b"200 User added successfully.\r\n").await?;

    info!("User {} added successfully with idents/ips: {:?}", username, idents_ips);
    Ok(())
}

// Add the is_valid_username function
fn is_valid_username(username: &str) -> bool {
    if username.len() < 1 || username.len() > 32 {
        return false;
    }
    if username == "rouilleftpd" {
        return false;
    }
    for c in username.chars() {
        if !c.is_ascii_alphanumeric() {
            return false;
        }
    }
    true
}

// Add the is_valid_ip_or_hostname function
fn is_valid_ip_or_hostname(ip: &str) -> bool {
    if ip.len() > 128 || ip.is_empty() {
        return false;
    }
    for c in ip.chars() {
        if !c.is_ascii_alphanumeric() && c != '.' && c != '-' && c != '@' {
            return false;
        }
    }
    true
}
