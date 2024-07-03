use crate::helpers::{sanitize_input, send_response};
use crate::session::Session;
use crate::Config;
use anyhow::Result;
use chrono::NaiveDateTime;
use filetime::{FileTime, set_file_times};
use log::{error, info};
use std::sync::Arc;
use tokio::net::TcpStream;
use tokio::sync::Mutex;

pub async fn handle_site_utime_command(
    writer: Arc<Mutex<TcpStream>>,
    _config: Arc<Config>,
    session: Arc<Mutex<Session>>,
    arg: String,
) -> Result<(), std::io::Error> {
    info!("Handling SITE UTIME command with arguments: {}", arg);

    let parts: Vec<&str> = arg.split_whitespace().collect();
    if parts.len() != 5 {
        send_response(&writer, b"501 Syntax error in parameters or arguments.\r\n").await?;
        return Ok(());
    }

    let file_path_arg = parts[0];
    let access_time_str = parts[1];
    let modify_time_str = parts[2];
    let create_time_str = parts[3];
    let _timezone = parts[4];  // "UTC" part, not used in this implementation

    info!("Parsed arguments - file_path: {}, access_time: {}, modify_time: {}, create_time: {}, timezone: {}",
          file_path_arg, access_time_str, modify_time_str, create_time_str, _timezone);

    // Sanitize the file path and remove any leading slashes
    let sanitized_file_path = sanitize_input(file_path_arg).trim_start_matches('/').to_string();
    info!("Sanitized file path: {}", sanitized_file_path);

    let file_path = {
        let session = session.lock().await;
        session.base_path.join(&sanitized_file_path)
    };

    if !file_path.exists() {
        error!("File not found: {}", file_path.display());
        send_response(&writer, b"550 File not found.\r\n").await?;
        return Ok(());
    }

    info!("Updating file times for: {}", file_path.display());

    // Parse the timestamps
    let access_time = NaiveDateTime::parse_from_str(access_time_str, "%Y%m%d%H%M%S")
        .map_err(|e| {
            error!("Invalid access time format: {}", e);
            std::io::Error::new(std::io::ErrorKind::InvalidInput, "Invalid access time")
        })?;
    let modify_time = NaiveDateTime::parse_from_str(modify_time_str, "%Y%m%d%H%M%S")
        .map_err(|e| {
            error!("Invalid modify time format: {}", e);
            std::io::Error::new(std::io::ErrorKind::InvalidInput, "Invalid modify time")
        })?;

    // Convert NaiveDateTime to FileTime
    let access_file_time = FileTime::from_unix_time(access_time.timestamp(), 0);
    let modify_file_time = FileTime::from_unix_time(modify_time.timestamp(), 0);

    // Update the file times
    if let Err(e) = set_file_times(&file_path, access_file_time, modify_file_time) {
        error!("Failed to update file times: {:?}", e);
        send_response(&writer, b"550 Failed to update file times.\r\n").await?;
        return Ok(());
    }

    send_response(&writer, b"200 Command okay.\r\n").await?;
    info!("File times updated successfully: {:?}", file_path);

    Ok(())
}
