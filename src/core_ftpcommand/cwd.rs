use std::sync::Arc;
use tokio::io::AsyncWriteExt;
use tokio::net::TcpStream;
use tokio::sync::Mutex;
use crate::Config;
use std::path::PathBuf;
use crate::core_network::Session;


pub async fn handle_cwd_command(
    writer: Arc<Mutex<TcpStream>>,
    config: Arc<Config>,
    session: Arc<Mutex<Session>>,
    arg: String,
) -> Result<(), std::io::Error> {
    let mut session = session.lock().await;
    let min_homedir = config.server.min_homedir.trim_start_matches('/');

    // Ensure the new directory is correctly handled as a relative path
    let new_dir = if arg.starts_with('/') {
        PathBuf::from(&arg)
    } else {
        PathBuf::from(&session.current_dir).join(&arg)
    };

    // Convert PathBuf to String, perform trimming, and then convert back to PathBuf
    let new_dir_str = new_dir.to_str().unwrap().trim_start_matches('/').to_string();
    let dir_path = PathBuf::from(&config.server.chroot_dir).join(min_homedir).join(new_dir_str);

    if dir_path.is_dir() {
        session.current_dir = new_dir.to_str().unwrap().to_string();
        let mut writer = writer.lock().await;
        writer.write_all(b"250 Directory successfully changed.\r\n").await?;
    } else {
        let mut writer = writer.lock().await;
        writer.write_all(b"550 Failed to change directory.\r\n").await?;
    }
    Ok(())
}
