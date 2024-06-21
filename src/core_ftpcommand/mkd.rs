use std::sync::Arc;
use tokio::io::AsyncWriteExt;
use tokio::net::TcpStream;
use tokio::sync::Mutex;
use crate::Config;
use std::path::PathBuf;
use tokio::fs;
use crate::core_network::Session;

pub async fn handle_mkd_command(
    writer: Arc<Mutex<TcpStream>>,
    config: Arc<Config>,
    session: Arc<Mutex<Session>>,
    arg: String,
) -> Result<(), std::io::Error> {
    let session = session.lock().await;
    let min_homedir = config.server.min_homedir.trim_start_matches('/');
    
    // Escape double quotes and sanitize input
    let sanitized_arg = arg.replace("\"", "\\\"");
    
    // Ensure the new directory is correctly handled as a relative path
    let new_dir = if sanitized_arg.starts_with('/') {
        PathBuf::from(&sanitized_arg)
    } else {
        PathBuf::from(&session.current_dir).join(&sanitized_arg)
    };

    // Convert PathBuf to String, perform trimming, and then convert back to PathBuf
    let new_dir_str = new_dir.to_str().unwrap().trim_start_matches('/').to_string();
    let dir_path = PathBuf::from(&config.server.chroot_dir).join(min_homedir).join(new_dir_str);

    // Create the directory
    match fs::create_dir_all(&dir_path).await {
        Ok(_) => {
            let mut writer = writer.lock().await;
            writer.write_all(format!("257 \"{}\" directory created.\r\n", sanitized_arg).as_bytes()).await?;
        },
        Err(_) => {
            let mut writer = writer.lock().await;
            writer.write_all(b"550 Failed to create directory.\r\n").await?;
        }
    }
    Ok(())
}
