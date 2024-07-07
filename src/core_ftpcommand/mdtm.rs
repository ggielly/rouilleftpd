use crate::helpers::{sanitize_input, send_response};
use crate::session::Session;
use crate::Config;
use chrono::NaiveDateTime;
use filetime::{FileTime};
use log::{error, info};

use filetime::set_file_times;
use std::sync::Arc;
use tokio::net::TcpStream;
use tokio::sync::Mutex;

pub async fn handle_mdtm_command(
    writer: Arc<Mutex<TcpStream>>,
    _config: Arc<Config>,
    session: Arc<Mutex<Session>>,
    arg: String,
) -> Result<(), std::io::Error> {
    info!("Handling MDTM command with arguments: {}", arg);

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

    let resolved_path = {
        let session = session.lock().await;
        let base_path = session.base_path.clone();
        let file_path = base_path.join(&sanitized_file_path);
        file_path.canonicalize().unwrap_or_else(|_| file_path)
    };

    if !resolved_path.exists() {
        error!("File not found: {}", resolved_path.display());
        send_response(&writer, b"550 File not found.\r\n").await?;
        return Ok(());
    }

    if !resolved_path.starts_with(&session.lock().await.base_path) {
        error!("Path is outside of the allowed area: {:?}", resolved_path);
        send_response(&writer, b"550 Path is outside of the allowed area.\r\n").await?;
        return Ok(());
    }

    info!("Updating file times for: {}", resolved_path.display());

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
    let access_file_time = FileTime::from_unix_time(access_time.and_utc().timestamp(), 0);
    let modify_file_time = FileTime::from_unix_time(modify_time.and_utc().timestamp(), 0);

    // Update the file times
    if let Err(e) = set_file_times(&resolved_path, access_file_time, modify_file_time) {
        error!("Failed to update file times: {:?}", e);
        send_response(&writer, b"550 Failed to update file times.\r\n").await?;
        return Ok(());
    }

    send_response(&writer, b"200 Command okay.\r\n").await?;
    info!("File times updated successfully: {:?}", resolved_path);

    Ok(())
}
