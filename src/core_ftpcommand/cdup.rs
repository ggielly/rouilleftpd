use crate::core_network::Session;
use crate::Config;
use log::{error, info, warn};
use std::path::Path;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::io::AsyncWriteExt;
use tokio::net::TcpStream;
use tokio::sync::Mutex;

pub async fn handle_cdup_command(
    writer: Arc<Mutex<TcpStream>>,
    config: Arc<Config>,
    session: Arc<Mutex<Session>>,
    _arg: String,
) -> Result<(), std::io::Error> {
    let mut session = session.lock().await;
    let current_dir = &session.current_dir;

    // Construct the new directory path by moving up one level
    let new_dir = PathBuf::from(current_dir)
        .parent()
        .unwrap_or_else(|| Path::new("/"))
        .to_path_buf();

    // Ensure the new directory is within the chroot environment
    let chroot_dir = PathBuf::from(&config.server.chroot_dir)
        .canonicalize()
        .unwrap();
    let new_dir_path = chroot_dir.join(new_dir.strip_prefix("/").unwrap_or(&new_dir));

    let canonical_new_dir_path = match new_dir_path.canonicalize() {
        Ok(path) => path,
        Err(_) => {
            error!(
                "Failed to canonicalize the directory path: {:?}",
                new_dir_path
            );
            let mut writer = writer.lock().await;
            writer
                .write_all(b"550 Failed to change directory.\r\n")
                .await?;
            return Ok(());
        }
    };

    if canonical_new_dir_path.starts_with(&chroot_dir) && canonical_new_dir_path.is_dir() {
        session.current_dir = canonical_new_dir_path
            .strip_prefix(&chroot_dir)
            .unwrap()
            .to_str()
            .unwrap()
            .to_string();
        if session.current_dir.is_empty() {
            session.current_dir = "/".to_string();
        } else {
            session.current_dir = format!("/{}", session.current_dir);
        }
        info!("Directory successfully changed to: {}", session.current_dir);
        let mut writer = writer.lock().await;
        writer
            .write_all(b"250 Directory successfully changed.\r\n")
            .await?;
    } else {
        warn!(
            "Failed to change directory to: {:?}",
            canonical_new_dir_path
        );
        let mut writer = writer.lock().await;
        writer
            .write_all(b"550 Failed to change directory.\r\n")
            .await?;
    }

    Ok(())
}
