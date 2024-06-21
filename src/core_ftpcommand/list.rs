use std::sync::Arc;
use tokio::io::AsyncWriteExt;
use tokio::net::TcpStream;
use tokio::sync::Mutex;
use crate::Config;
use crate::core_network::Session;
use std::fs;
use std::path::PathBuf;
use log::{info, warn, error};

pub async fn handle_list_command(
    writer: Arc<Mutex<TcpStream>>,
    config: Arc<Config>,
    session: Arc<Mutex<Session>>,
    _arg: String,
) -> Result<(), std::io::Error> {
    let session = session.lock().await;
    let current_dir = &session.current_dir;
    let min_homedir = config.server.min_homedir.trim_start_matches('/');
    let dir_path = PathBuf::from(&config.server.chroot_dir).join(min_homedir).join(current_dir.trim_start_matches('/'));

    info!("chroot_dir: {:?}", config.server.chroot_dir);
    info!("min_homedir: {:?}", config.server.min_homedir);
    info!("Current dir: {:?}", current_dir);
    info!("Constructed directory path: {:?}", dir_path);

    let entries = match fs::read_dir(&dir_path) {
        Ok(entries) => entries,
        Err(e) => {
            error!("Error reading directory: {:?}", e);
            error!("Real path attempted: {:?}", dir_path.canonicalize());
            let mut writer = writer.lock().await;
            writer.write_all(b"550 Failed to list directory.\r\n").await?;
            return Ok(());
        }
    };

    let mut listing = String::new();

    for entry in entries {
        if let Ok(entry) = entry {
            let metadata = match entry.metadata() {
                Ok(metadata) => metadata,
                Err(e) => {
                    warn!("Failed to get metadata for entry: {:?}, error: {:?}", entry.path(), e);
                    continue;
                },
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

    let mut writer = writer.lock().await;
    writer.write_all(b"150 Here comes the directory listing.\r\n").await?;
    writer.write_all(listing.as_bytes()).await?;
    writer.write_all(b"226 Directory send OK.\r\n").await?;
    info!("Directory listing sent successfully.");
    Ok(())
}
