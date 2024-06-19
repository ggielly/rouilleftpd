// src/core_ftpcommand/pass.rs
use std::sync::Arc;
use tokio::io::AsyncWriteExt;
use tokio::net::TcpStream;
use tokio::sync::Mutex;
use crate::Config;

pub async fn handle_pass_command(
    writer: Arc<Mutex<TcpStream>>,
    _config: Arc<Config>,
    _password: String,
) -> Result<(), std::io::Error> {
    let mut writer = writer.lock().await;
    writer.write_all(b"230 User logged in, proceed.\r\n").await?;
    Ok(())
}
