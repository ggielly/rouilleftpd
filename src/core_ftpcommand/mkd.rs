use std::sync::Arc;
use tokio::io::AsyncWriteExt;
use tokio::net::TcpStream;
use tokio::sync::Mutex;
use crate::Config;
use std::path::{Path, PathBuf};
use tokio::fs;
use crate::core_network::Session;
use anyhow::Result;

/// Handles the MKD (Make Directory) FTP command.
///
/// This function creates a new directory on the server within the user's current directory.
/// The function ensures the new directory is within the allowed chroot area, sanitizes inputs to
/// prevent directory traversal attacks, and sends appropriate responses back to the FTP client.
///
/// # Arguments
///
/// * `writer` - A shared, locked TCP stream for writing responses to the client.
/// * `config` - A shared server configuration.
/// * `session` - A shared, locked session containing the user's current state.
/// * `arg` - The directory name to create.
///
/// # Returns
///
/// Result<(), std::io::Error> indicating the success or failure of the operation.
pub async fn handle_mkd_command(
    writer: Arc<Mutex<TcpStream>>,
    config: Arc<Config>,
    session: Arc<Mutex<Session>>,
    arg: String,
) -> Result<(), std::io::Error> {
    // Sanitize the input argument to prevent directory traversal attacks.
    let sanitized_arg = sanitize_input(&arg);

    // Construct the new directory path within the user's current directory.
    let (new_dir, dir_path) = {
        // Lock the session to get the current directory.
        let session = session.lock().await;
        construct_paths(&config, &session.current_dir, &sanitized_arg)
    };

    // Canonicalize the chroot directory to resolve any symbolic links or relative paths.
    let chroot_dir = PathBuf::from(&config.server.chroot_dir).canonicalize()?;
    // Canonicalize the new directory path to ensure it's within the chroot directory.
    let resolved_path = dir_path.canonicalize().unwrap_or_else(|_| dir_path.clone());

    // Check if the resolved path is within the chroot directory.
    if !resolved_path.starts_with(&chroot_dir) {
        send_response(&writer, b"550 Path is outside of the allowed area.\r\n").await?;
        return Ok(());
    }

    // Check if the directory already exists.
    if resolved_path.exists() {
        send_response(&writer, b"550 Directory already exists.\r\n").await?;
        return Ok(());
    }

    // Attempt to create the directory.
    match fs::create_dir_all(&resolved_path).await {
        Ok(_) => {
            // Send success response if the directory was created successfully.
            send_response(&writer, format!("257 \"{}\" directory created.\r\n", sanitized_arg).as_bytes()).await?;
        },
        Err(_) => {
            // Send failure response if there was an error creating the directory.
            send_response(&writer, b"550 Failed to create directory.\r\n").await?;
        }
    }

    Ok(())
}

/// Sanitizes the input argument to prevent directory traversal attacks.
fn sanitize_input(arg: &str) -> String {
    arg.trim_start_matches('/').replace("\"", "\\\"")
}

/// Constructs the new directory path within the user's current directory and the server's chroot directory.
fn construct_paths(config: &Config, current_dir: &str, sanitized_arg: &str) -> (PathBuf, PathBuf) {
    // Join the current directory with the sanitized argument to form the new directory path.
    let new_dir = PathBuf::from(current_dir).join(sanitized_arg);
    // Convert the new directory path to a string, trimming leading slashes.
    let new_dir_str = new_dir.to_str().unwrap().trim_start_matches('/').to_string();
    // Construct the full path within the chroot directory.
    let dir_path = PathBuf::from(&config.server.chroot_dir)
        .join(config.server.min_homedir.trim_start_matches('/'))
        .join(new_dir_str);

    (new_dir, dir_path)
}

/// Sends a response message to the client via the writer.
async fn send_response(writer: &Arc<Mutex<TcpStream>>, message: &[u8]) -> Result<(), std::io::Error> {
    let mut writer = writer.lock().await;
    writer.write_all(message).await?;
    writer.flush().await?;
    Ok(())
}
