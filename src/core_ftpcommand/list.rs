use crate::Config;
use std::fs;
use std::io;
use std::path::Path;
use std::sync::Arc;
use tokio::io::AsyncWriteExt;
use tokio::net::TcpStream;
use tokio::sync::Mutex;

pub async fn handle_list_command(
    writer: Arc<Mutex<TcpStream>>,
    _config: Arc<Config>,
    _arg: String,
) -> Result<(), std::io::Error> {
    // Define the directory to list. For simplicity, we use the current directory.
    let dir = ".";
    let path = Path::new(dir);

    // Read the directory contents.
    let entries = match fs::read_dir(path) {
        Ok(entries) => entries,
        Err(_) => {
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
                Err(_) => continue,
            };

            let file_type = if metadata.is_dir() { "d" } else { "-" };
            let permissions = metadata.permissions();

            // Dummy values for owner, group, size, and date.
            let owner = "owner";
            let group = "group";
            let size = metadata.len();
            let date = "Jan 01 00:00";

            // Format: type permissions link_count owner group size date name
            let file_name = entry.file_name().into_string().unwrap_or_default();
            let file_entry = format!(
                "{}rwxr-xr-x 1 {} {} {} {} {}\r\n",
                file_type, owner, group, size, date, file_name
            );

            listing.push_str(&file_entry);
        }
    }

    let mut writer = writer.lock().await;
    writer
        .write_all(b"150 Here comes the directory listing.\r\n")
        .await?;
    writer.write_all(listing.as_bytes()).await?;
    writer.write_all(b"226 Directory send OK.\r\n").await?;
    Ok(())
}
