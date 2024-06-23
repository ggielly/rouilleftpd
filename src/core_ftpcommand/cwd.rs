use crate::session::Session;
use crate::Config;
use log::{error, info, warn};
use std::path::{Component, PathBuf};
use std::sync::Arc;
use tokio::io::AsyncWriteExt;
use tokio::net::TcpStream;
use tokio::sync::Mutex;

pub async fn handle_cwd_command(
    writer: Arc<Mutex<TcpStream>>,
    _config: Arc<Config>,
    session: Arc<Mutex<Session>>,
    arg: String,
) -> Result<(), std::io::Error> {
    let mut session = session.lock().await;
    let sanitized_arg = arg.replace("\"", "\\\"");
    info!("Received CWD command with argument: {}", sanitized_arg);

    // Ensure the new directory is correctly handled as a relative path
    let new_dir = if sanitized_arg.starts_with('/') {
        PathBuf::from(&sanitized_arg)
    } else {
        PathBuf::from(&session.current_dir).join(&sanitized_arg)
    };

    // Normalize the path to handle ".." and other relative components
    let new_dir_normalized = new_dir.components().fold(PathBuf::new(), |mut acc, comp| {
        match comp {
            Component::ParentDir => {
                acc.pop();
            }
            Component::RootDir => acc.push("/"),
            Component::Normal(part) => acc.push(part),
            _ => {}
        }
        acc
    });

    // Construct the full path within the chroot environment
    let full_path = session.base_path.join(
        new_dir_normalized
            .strip_prefix("/")
            .unwrap_or(&new_dir_normalized),
    );

    // Log the constructed directory path
    info!("Constructed directory path: {:?}", full_path);

    // Validate the final directory path is within the chroot directory
    let canonical_dir_path = match full_path.canonicalize() {
        Ok(path) => path,
        Err(_) => {
            error!("Failed to canonicalize the directory path: {:?}", full_path);
            let mut writer = writer.lock().await;
            writer
                .write_all(b"550 Failed to change directory.\r\n")
                .await?;
            return Ok(());
        }
    };

    if canonical_dir_path.starts_with(&session.base_path) && canonical_dir_path.is_dir() {
        session.current_dir = format!(
            "/{}",
            canonical_dir_path
                .strip_prefix(&session.base_path)
                .unwrap()
                .to_str()
                .unwrap()
        );
        info!("Directory successfully changed to: {}", session.current_dir);
        let mut writer = writer.lock().await;
        writer
            .write_all(b"250 Directory successfully changed.\r\n")
            .await?;
    } else {
        warn!("Failed to change directory to: {:?}", canonical_dir_path);
        let mut writer = writer.lock().await;
        writer
            .write_all(b"550 Failed to change directory.\r\n")
            .await?;
    }
    Ok(())
}
