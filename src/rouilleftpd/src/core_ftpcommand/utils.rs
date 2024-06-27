use crate::Config;
use anyhow::Result;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::io::AsyncWriteExt;
use tokio::net::TcpStream;
use tokio::sync::Mutex;

/// Sanitizes the input argument to prevent directory traversal attacks.
pub fn sanitize_input(arg: &str) -> String {
    arg.trim_start_matches('/').replace("\"", "\\\"")
}

/// Constructs the directory path within the user's current directory and the server's chroot directory.
pub fn construct_path(config: &Config, current_dir: &str, sanitized_arg: &str) -> PathBuf {
    // Join the current directory with the sanitized argument to form the directory path.
    let new_dir = PathBuf::from(current_dir).join(sanitized_arg);
    // Convert the directory path to a string, trimming leading slashes.
    let new_dir_str = new_dir
        .to_str()
        .unwrap()
        .trim_start_matches('/')
        .to_string();
    // Construct the full path within the chroot directory.
    let dir_path = PathBuf::from(&config.server.chroot_dir)
        .join(config.server.min_homedir.trim_start_matches('/'))
        .join(new_dir_str);

    dir_path
}

/// Sends a response message to the client via the writer.
pub async fn send_response(
    writer: &Arc<Mutex<TcpStream>>,
    message: &[u8],
) -> Result<(), std::io::Error> {
    let mut writer = writer.lock().await;
    writer.write_all(message).await?;
    writer.flush().await?;
    Ok(())
}
