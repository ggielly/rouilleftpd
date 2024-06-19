use tokio::io::{AsyncWriteExt, Result};
use tokio::net::TcpStream;
use tokio::sync::Mutex;
use std::sync::Arc;
use crate::Config;

pub async fn handle_pass_command(writer: Arc<Mutex<TcpStream>>, _config: Arc<Config>, _password: String) -> Result<()> {
    let mut writer = writer.lock().await;
    writer.write_all(b"230 User logged in, proceed.\r\n").await?;
    Ok(())
}
