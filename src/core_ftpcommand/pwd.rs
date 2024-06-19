// src/core_ftpcommand/pwd.rs
use std::sync::Arc;
use tokio::io::AsyncWriteExt;
use tokio::net::TcpStream;
use tokio::sync::Mutex;
use crate::Config;

pub async fn handle_pwd_command(
    writer: Arc<Mutex<TcpStream>>,
    config: Arc<Config>,
    _arg: String,
) -> Result<(), std::io::Error> {
    let mut writer = writer.lock().await;

    // Get the min home directory from the config
    let min_homedir = &config.server.min_homedir;

    // If the min_homedir is "/", return it directly
    let response = if min_homedir == "/" {
        format!("257 \"/\" is the current directory.\r\n")
    } else {
        // Otherwise, return the min_homedir with quotes
        format!("257 \"{}\" is the current directory.\r\n", min_homedir)
    };

    writer.write_all(response.as_bytes()).await?;
    Ok(())
}
