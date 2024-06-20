use std::sync::Arc;
use tokio::io::AsyncWriteExt;
use tokio::net::TcpStream;
use tokio::sync::Mutex;
use crate::Config;

pub async fn handle_noop_command(
    writer: Arc<Mutex<TcpStream>>,
    _config: Arc<Config>,
    _arg: String,
) -> Result<(), std::io::Error> {
    let mut writer = writer.lock().await;
    writer.write_all(b"200 OK, n00p n00p !\r\n").await?;
    Ok(())
}
