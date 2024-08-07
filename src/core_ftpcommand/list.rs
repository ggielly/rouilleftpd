use crate::session::Session;
use crate::Config;
use log::{debug, error, info, warn};
use std::fs;
use std::sync::Arc;

use tokio::io::AsyncWriteExt;
use tokio::net::TcpStream;
use tokio::sync::Mutex;

use crate::helpers::generate_and_send_statline;

pub async fn handle_list_command(
    writer: Arc<Mutex<TcpStream>>,
    config: Arc<Config>,
    session: Arc<Mutex<Session>>,
    _arg: String,
) -> Result<(), std::io::Error> {
    let session_lock = session.lock().await;
    let current_dir = &session_lock.current_dir;
    let dir_path = session_lock
        .base_path
        .join(current_dir.trim_start_matches('/'));

    debug!("base_path: {:?}", session_lock.base_path);
    debug!("Current dir: {:?}", current_dir);
    debug!("Constructed directory path: {:?}", dir_path);

    let canonical_dir_path = match dir_path.canonicalize() {
        Ok(path) => path,
        Err(e) => {
            error!("Failed to canonicalize the directory path: {:?}", e);
            let mut writer = writer.lock().await;
            writer
                .write_all(b"550 Failed to list directory.\r\n")
                .await?;
            return Ok(());
        }
    };

    if !canonical_dir_path.starts_with(&session_lock.base_path) {
        warn!(
            "Directory listing attempt outside chroot: {:?}",
            canonical_dir_path
        );
        let mut writer = writer.lock().await;
        writer
            .write_all(b"550 Failed to list directory.\r\n")
            .await?;
        return Ok(());
    }

    let entries = match fs::read_dir(&canonical_dir_path) {
        Ok(entries) => entries,
        Err(e) => {
            error!("Error reading directory: {:?}", e);
            error!("Real path attempted: {:?}", canonical_dir_path);
            let mut writer = writer.lock().await;
            writer
                .write_all(b"550 Failed to list directory.\r\n")
                .await?;
            return Ok(());
        }
    };

    let mut listing = String::new();

    for entry in entries {
        if let Ok(entry) = entry {
            let metadata = match entry.metadata() {
                Ok(metadata) => metadata,
                Err(e) => {
                    warn!(
                        "Failed to get metadata for entry: {:?}, error: {:?}",
                        entry.path(),
                        e
                    );
                    continue;
                }
            };

            let file_type = if metadata.is_dir() { "d" } else { "-" };

            let owner = "owner";
            let group = "group";
            let size = metadata.len();
            let date = "Jan 01 00:00";

            let file_name = entry.file_name().into_string().unwrap_or_default();
            let file_entry = format!(
                "{}rwxr-xr-x 1 {} {} {} {} {}\r\n",
                file_type, owner, group, size, date, file_name
            );

            listing.push_str(&file_entry);
        } else {
            warn!("Failed to read directory entry.");
        }
    }

    let data_stream = session_lock.data_stream.clone(); // Clone the data stream Arc<Mutex<TcpStream>>

    drop(session_lock); // Explicitly drop the session lock before using the data stream

    if let Some(data_stream) = data_stream {
        let mut data_stream = data_stream.lock().await;
        let mut writer = writer.lock().await;
        writer
            .write_all(b"150 Here comes the directory listing.\r\n")
            .await?;

        match data_stream.write_all(listing.as_bytes()).await {
            Ok(_) => {
                if let Err(e) = data_stream.shutdown().await {
                    error!("Failed to shutdown data stream: {:?}", e);
                }
                writer.write_all(b"226 Directory send OK.\r\n").await?;
                info!("Directory listing sent successfully.");
            }
            Err(e) => {
                error!("Failed to send directory listing: {:?}", e);
                writer
                    .write_all(b"426 Connection closed; transfer aborted.\r\n")
                    .await?;
            }
        }
    } else {
        let writer_info = format!("{:?}", writer);
        let config_info = format!("{:?}", config);
        let session_info = format!("{:?}", session);

        error!(
            "Failed to open data connection for directory listing. 
            Writer: {}, 
            Config: {}, 
            Session: {}",
            writer_info, config_info, session_info
        );

        let mut writer = writer.lock().await;
        writer
            .write_all(b"425 Can't open data connection.\r\n")
            .await?;
    }

    generate_and_send_statline(writer.clone(), config.clone(), session.clone()).await?;

    Ok(())
}
