use crate::session::Session;
use crate::Config;
use log::{error, info};
use std::sync::Arc;
use tokio::io::AsyncWriteExt;
use tokio::net::TcpStream;
use tokio::sync::Mutex;

pub async fn handle_pwd_command(
    writer: Arc<Mutex<TcpStream>>,
    _config: Arc<Config>,
    session: Arc<Mutex<Session>>,
    _arg: String,
) -> Result<(), std::io::Error> {
    // Lock the session to get the current directory.
    let session = session.lock().await;
    let current_dir = &session.current_dir;

    // Ensure the current_dir starts with a '/'
    let response_path = if current_dir.starts_with('/') {
        current_dir.to_string()
    } else {
        format!("/{}", current_dir)
    };

    let response = format!("257 \"{}\" is the current directory.\r\n", response_path);

    info!(
        "Responding to PWD command with current directory: {}",
        response_path
    );

    // Lock the writer to send the response.
    let mut writer = writer.lock().await;
    if let Err(e) = writer.write_all(response.as_bytes()).await {
        error!("Failed to send PWD response: {}", e);
        return Err(e);
    }

    Ok(())
}
