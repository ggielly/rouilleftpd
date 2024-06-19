use tokio::io::{AsyncWriteExt, Result};
use tokio::net::TcpStream;
use tokio::sync::Mutex;
use std::sync::Arc;
use crate::Config;

pub async fn handle_pwd_command(writer: Arc<Mutex<TcpStream>>, config: Arc<Config>, _arg: String) -> Result<()> {
    let current_dir = &config.server.chroot_dir;
    let response = format!("257 \"{}\" is the current directory.\r\n", current_dir);
    let mut writer = writer.lock().await;
    writer.write_all(response.as_bytes()).await?;
    Ok(())
}
