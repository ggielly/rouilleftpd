use crate::helpers::{sanitize_input, send_response};
use crate::session::Session;
use crate::Config;
use chrono::NaiveDateTime;
use filetime::{set_file_mtime, FileTime};
use log::{error, info, warn};
use std::fs;
use std::io::{self, ErrorKind};
use std::sync::Arc;
use tokio::io::Result as TokioResult;
use tokio::net::TcpStream;
use tokio::sync::Mutex;

pub async fn handle_mdtm_command(
    writer: Arc<Mutex<TcpStream>>,
    _config: Arc<Config>,
    session: Arc<Mutex<Session>>,
    arg: String,
) -> TokioResult<()> {
    if arg.trim().is_empty() {
        warn!("MDTM command received with no arguments");
        send_response(&writer, b"501 Syntax error in parameters or arguments.\r\n").await?;
        return Ok(());
    }

    let parts: Vec<&str> = arg.split_whitespace().collect();
    if parts.len() == 1 {
        // Retrieve the modification time
        let filename = sanitize_input(parts[0]);
        info!(
            "Received MDTM command to get modification time for file: {}",
            filename
        );

        let resolved_path = {
            let session = session.lock().await;
            let base_path = session.base_path.clone();
            let file_path = base_path.join(&filename);
            file_path.canonicalize().unwrap_or_else(|_| file_path)
        };

        if !resolved_path.exists() {
            error!("File not found: {:?}", resolved_path);
            send_response(&writer, b"550 File not found.\r\n").await?;
            return Ok(());
        }

        let metadata = fs::metadata(&resolved_path).map_err(|e| {
            error!(
                "Failed to retrieve metadata for file: {:?}, error: {}",
                resolved_path, e
            );
            e
        })?;

        let modified_time = FileTime::from_last_modification_time(&metadata);
        let modified_time = NaiveDateTime::from_timestamp(modified_time.unix_seconds(), 0);
        let response = format!("213 {}\r\n", modified_time.format("%Y%m%d%H%M%S"));

        send_response(&writer, response.as_bytes()).await?;
    } else if parts.len() == 2 {
        // Set the modification time
        let datetime_str = parts[0];
        let filename = sanitize_input(parts[1]);
        info!(
            "Received MDTM command to set modification time for file: {} to {}",
            filename, datetime_str
        );

        let datetime =
            NaiveDateTime::parse_from_str(datetime_str, "%Y%m%d%H%M%S").map_err(|e| {
                error!("Invalid datetime format: {}", e);
                io::Error::new(ErrorKind::InvalidInput, "Invalid datetime format")
            })?;

        let resolved_path = {
            let session = session.lock().await;
            let base_path = session.base_path.clone();
            let file_path = base_path.join(&filename);
            file_path.canonicalize().unwrap_or_else(|_| file_path)
        };

        if !resolved_path.exists() {
            error!("File not found: {:?}", resolved_path);
            send_response(&writer, b"550 File not found.\r\n").await?;
            return Ok(());
        }

        let filetime = FileTime::from_unix_time(datetime.and_utc().timestamp(), 0);
        set_file_mtime(&resolved_path, filetime).map_err(|e| {
            error!(
                "Failed to set modification time for file: {:?}, error: {}",
                resolved_path, e
            );
            e
        })?;

        send_response(&writer, b"213 Modification time set.\r\n").await?;
    } else {
        warn!("MDTM command received with invalid arguments: {}", arg);
        send_response(&writer, b"501 Syntax error in parameters or arguments.\r\n").await?;
    }

    Ok(())
}
