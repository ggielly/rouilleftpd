use tokio::io::AsyncWriteExt;
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::net::TcpStream;

// Make respond_with_* functions public so that they can be called from other modules.
pub async fn respond_with_error(writer: &Arc<Mutex<TcpStream>>, msg: &[u8]) -> Result<(), std::io::Error> {
    let mut writer = writer.lock().await;
    writer.write_all(msg).await
}

pub async fn respond_with_success(writer: &Arc<Mutex<TcpStream>>, msg: &[u8]) -> Result<(), std::io::Error> {
    let mut writer = writer.lock().await;
    writer.write_all(msg).await
}
