use std::sync::Arc;
use tokio::io::AsyncWriteExt;
use tokio::net::TcpStream;
use tokio::sync::Mutex;
use crate::Config;
use crate::core_network::Session;

pub async fn handle_pwd_command(
    writer: Arc<Mutex<TcpStream>>,
    _config: Arc<Config>,
    session: Arc<Mutex<Session>>,
    _arg: String,
) -> Result<(), std::io::Error> {
    let session = session.lock().await;
    let current_dir = &session.current_dir;
    let response = format!("257 \"{}\" is the current directory.\r\n", current_dir);

    let mut writer = writer.lock().await;
    writer.write_all(response.as_bytes()).await?;
    Ok(())
}
