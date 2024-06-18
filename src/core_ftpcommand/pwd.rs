// src/core_ftpcommand/pwd.rs
use tokio::io::AsyncWriteExt;
use tokio::net::TcpStream;
use tokio::sync::Mutex;
use std::sync::Arc;
use crate::Config;

pub async fn handle_pwd_command(writer: Arc<Mutex<TcpStream>>, config: Arc<Config>) -> std::io::Result<()> {
    let current_dir = &config.server.chroot_dir;
    let response = format!("257 \"{}\" is the current directory.\r\n", current_dir);

    // On Windows systems, the path will be formatted with Windows style separators ('\')
    // Most FTP clients expect normal UNIX separators ('/'), so we replace the separators here.
    #[cfg(windows)]
    let response = response.replace(std::path::MAIN_SEPARATOR, "/");

    let mut writer = writer.lock().await;
    writer.write_all(response.as_bytes()).await?;
    Ok(())
}
