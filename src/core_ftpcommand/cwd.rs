use std::sync::Arc;
use tokio::io::AsyncWriteExt;
use tokio::net::TcpStream;
use tokio::sync::Mutex;
use crate::{Config, Session};
use std::path::PathBuf;

pub async fn handle_cwd_command(
    writer: Arc<Mutex<TcpStream>>,
    config: Arc<Config>,
    session: Arc<Mutex<Session>>,
    arg: String,
) -> Result<(), std::io::Error> {
    let mut session = session.lock().await;
    let min_homedir = config.server.min_homedir.trim_start_matches('/');
    let new_dir = if arg.starts_with('/') {
        PathBuf::from(&arg)
    } else {
        PathBuf::from(&session.current_dir).join(&arg)
    };
    let dir_path = PathBuf::from(&config.server.chroot_dir).join(min_homedir).join(new_dir.trim_start_matches('/'));

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
