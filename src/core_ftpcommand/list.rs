use crate::session::Session;
use crate::Config;
use log::{error, info, warn};
use std::fs;
use std::sync::Arc;
use tokio::io::AsyncWriteExt;
use tokio::net::TcpStream;
use tokio::sync::Mutex;

pub async fn handle_list_command(
    writer: Arc<Mutex<TcpStream>>,
    _config: Arc<Config>,
    session: Arc<Mutex<Session>>,
    _arg: String,
) -> Result<(), std::io::Error> {
    let session = session.lock().await;
    let current_dir = &session.current_dir;
    let dir_path = session.base_path.join(current_dir.trim_start_matches('/'));

    info!("base_path: {:?}", session.base_path);
    info!("Current dir: {:?}", current_dir);
    info!("Constructed directory path: {:?}", dir_path);

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

    if !canonical_dir_path.starts_with(&session.base_path) {
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

    let data_stream = session.data_stream.clone(); // Clone the data stream Arc<Mutex<TcpStream>>

    drop(session); // Explicitly drop the session lock before using the data stream

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
        let mut writer = writer.lock().await;
        writer
            .write_all(b"425 Can't open data connection.\r\n")
            .await?;
    }

    Ok(())
}
