use tokio::io::{AsyncWriteExt, Result};
use tokio::net::TcpStream;
use tokio::sync::Mutex;
use std::sync::Arc;
use crate::Config;

pub async fn handle_quit_command(writer: Arc<Mutex<TcpStream>>, _config: Arc<Config>, _arg: String) -> Result<()> {
    let mut writer = writer.lock().await;
    writer.write_all(b"221 Service closing control connection.\r\n").await?;
    Ok(())
}
