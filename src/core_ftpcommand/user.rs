use tokio::io::{AsyncWriteExt, Result};
use tokio::net::TcpStream;
use tokio::sync::Mutex;
use std::sync::Arc;
use crate::Config;

pub async fn handle_user_command(writer: Arc<Mutex<TcpStream>>, _config: Arc<Config>, username: String) -> Result<()> {
    let mut writer = writer.lock().await;
    writer.write_all(b"331 User name okay, need password.\r\n").await?;
    Ok(())
}