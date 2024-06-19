// src/core_ftpcommand/user.rs
use std::sync::Arc;
use tokio::io::AsyncWriteExt;
use tokio::net::TcpStream;
use tokio::sync::Mutex;
use crate::Config;

pub async fn handle_user_command(
    writer: Arc<Mutex<TcpStream>>,
    _config: Arc<Config>,
    _username: String,
) -> Result<(), std::io::Error> {
    let mut writer = writer.lock().await;
    writer.write_all(b"331 User name okay, need password.\r\n").await?;
    Ok(())
}
