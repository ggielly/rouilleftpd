use std::sync::Arc;
use tokio::io::AsyncWriteExt;
use tokio::net::TcpStream;
use tokio::sync::Mutex;
use crate::Config;
use crate::core_network::Session;
use log::{info, error};

pub async fn handle_pwd_command(
    writer: Arc<Mutex<TcpStream>>,
    _config: Arc<Config>,
    session: Arc<Mutex<Session>>,
    _arg: String,
) -> Result<(), std::io::Error> {
    // Lock the session to get the current directory.
    let session = session.lock().await;
    let current_dir = &session.current_dir;

    let response = format!("257 \"{}\" is the current directory.\r\n", current_dir);

    info!("Responding to PWD command with current directory: {}", current_dir);

    // Lock the writer to send the response.
    let mut writer = writer.lock().await;
    if let Err(e) = writer.write_all(response.as_bytes()).await {
        error!("Failed to send PWD response: {}", e);
        return Err(e);
    }

    Ok(())
}
