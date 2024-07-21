use crate::{Config, Ipc};

use anyhow::{Context, Result};

use crate::core_ftpcommand::site::helper::cleanup_cookie_statline;
use crate::session::Session;
use crate::users::update_user_record;
use log::{debug, error, info};
use std::collections::HashMap;
use std::fs;
use std::io::Result as IoResult;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::io::AsyncWriteExt;
use tokio::net::TcpStream;
use tokio::sync::Mutex;

use crate::constants::STATLINE_PATH;

use std::path::Path;
use sysinfo::{DiskExt, System, SystemExt};

pub fn pad_message(message: &[u8], length: usize) -> Vec<u8> {
    let mut padded_message = Vec::with_capacity(length);
    padded_message.extend_from_slice(message);
    while padded_message.len() < length {
        padded_message.push(b' '); // Pad with spaces up to the desired length
    }
    padded_message
}

/// Sanitizes input to prevent directory traversal attacks and ensure paths are relative.
pub fn sanitize_input(input: &str) -> String {
    // Remove directory traversal sequences
    let sanitized_input = input.replace("../", "").replace("..\\", "");
    // Remove any leading slashes
    sanitized_input.trim_start_matches('/').to_string()
}
/// Sends a response to the client.
pub async fn send_response(
    writer: &Arc<Mutex<TcpStream>>,
    message: &[u8],
) -> Result<(), std::io::Error> {
    let mut writer = writer.lock().await;
    writer.write_all(message).await?;
    Ok(())
}

// Example function that handles a command
pub async fn handle_command(
    ipc: &Ipc,
    username: &str,
    command: &str,
    download_speed: f32,
    upload_speed: f32,
) {
    // Update the user record in shared memory
    update_user_record(ipc, username, command, download_speed, upload_speed);
}

pub fn load_config(path: &str) -> Result<Config> {
    let config_str = fs::read_to_string(path)
        .map_err(|e| anyhow::Error::new(e))
        .with_context(|| format!("Failed to read configuration file: {}", path))?;
    let config: Config = toml::from_str(&config_str)
        .with_context(|| format!("Failed to parse configuration file: {}", path))?;

    eprintln!("Loaded config: {:?}", config);

    Ok(config)
}
async fn read_config(path: &str) -> Result<String> {
    let config_str = tokio::fs::read_to_string(path)
        .await
        .map_err(|e| anyhow::Error::new(e))
        .with_context(|| format!("Failed to read configuration file: {}", path))?;
    Ok(config_str)
}

// Helper function to log configuration options
pub fn log_config(config: &Config) {
    info!("  Listen Port: {}", config.server.listen_port);
    info!("  PASV Address: {}", config.server.pasv_address);
    info!("  IPC Key: {}", config.server.ipc_key);
    info!("  Chroot Directory: {}", config.server.chroot_dir);
    info!("  Minimum Home Directory: {}", config.server.min_homedir);
    info!(
        "  Upload Buffer Size: {} KB",
        config.server.upload_buffer_size.unwrap_or(256 * 1024) / 1024
    );
    info!(
        "  Download Buffer Size: {} KB",
        config.server.download_buffer_size.unwrap_or(256 * 1024) / 1024
    );
}

pub fn load_banner(path: &str) -> Result<String> {
    // Attempt to read the file contents directly
    let config_str = fs::read_to_string(path)
        .map_err(|e| {
            error!("Failed to read banner file: {}: {}", path, e);
            anyhow::Error::new(e)
        })
        .with_context(|| format!("Failed to read banner file: {}", path))?;

    // Check if the file is empty
    if config_str.is_empty() {
        error!("Banner file is empty: {}", path);
        return Err(anyhow::Error::msg("Banner file is empty."));
    }

    info!("Banner file loaded successfully: {}", path);
    Ok(config_str)
}

pub fn load_file(path: &str) -> Result<String, anyhow::Error> {
    let file_content = fs::read_to_string(path)
        .map_err(|e| {
            error!("Failed to read file: {}: {}", path, e);
            anyhow::Error::new(e)
        })
        .with_context(|| format!("Failed to read file: {}", path))?;

    if file_content.is_empty() {
        error!("File is empty: {}", path);
        return Err(anyhow::Error::msg("The file is empty."));
    }

    info!("File loaded successfully: {}", path);
    Ok(file_content)
}

pub async fn send_file_to_client(
    writer: &Arc<Mutex<TcpStream>>,
    chroot_dir: &str,
    file_path: &str,
) -> IoResult<()> {
    let full_path = PathBuf::from(chroot_dir).join(file_path);
    match load_file(full_path.to_str().unwrap()) {
        Ok(content) => {
            let mut writer = writer.lock().await;
            for line in content.lines() {
                let formatted_line = format!("200- {}\r\n", line);
                writer.write_all(formatted_line.as_bytes()).await?;
            }
            let end_message = "200- \r\n";
            writer.write_all(end_message.as_bytes()).await?;
            writer.flush().await?;
            Ok(())
        }
        Err(e) => {
            error!("Failed to load file: {}", e);
            let error_message = format!("500 Internal server error: {}\r\n", e);
            let mut writer = writer.lock().await;
            writer.write_all(error_message.as_bytes()).await?;
            writer.flush().await?;
            Ok(())
        }
    }
}

pub fn get_site_free_space(path: &Path) -> Result<u64, String> {
    // Create a System object to retrieve system information
    let mut sys = System::new_all();
    sys.refresh_all();

    // Find the disk that contains the given path
    for disk in sys.disks() {
        if path.starts_with(disk.mount_point()) {
            return Ok(disk.available_space());
        }
    }

    // If no matching disk is found, return an error
    Err(format!("No disk found containing the path: {:?}", path))
}

pub fn format_free_space(size_in_mb: f64) -> String {
    if size_in_mb >= 1_048_576.0 {
        format!("{:.2} TB", size_in_mb / 1_048_576.0)
    } else if size_in_mb >= 1_024.0 {
        format!("{:.2} GB", size_in_mb / 1_024.0)
    } else {
        format!("{:.2} MB", size_in_mb)
    }
}

async fn load_statline(config: Arc<Config>) -> Result<String> {
    let path = format!("{}{}", config.server.chroot_dir, STATLINE_PATH);

    let statline = std::fs::read_to_string(&path)
        .with_context(|| format!("Failed to read statline file: {}", path))
        .map_err(|e| {
            error!("Error reading statline file {}: {}", path, e);
            e
        })?;

    if statline.is_empty() {
        error!("Statline file is empty: {}", path);
        return Err(anyhow::Error::msg("Statline file is empty"));
    }

    Ok(statline)
}

pub async fn generate_and_send_statline(
    writer: Arc<Mutex<TcpStream>>,
    config: Arc<Config>,
    session: Arc<Mutex<Session>>,
) -> Result<(), std::io::Error> {
    let statline_template = load_statline(config.clone()).await.unwrap_or_default();

    // Initialize the replacements map
    let mut replacements = HashMap::new();

    // Example statline template placeholders
    replacements.insert("%[%.1f]IG%[%s]Y", format!("{:.1}MB", 0.0));
    replacements.insert("%[%.1f]IJ%[%s]Y", format!("{:.1}MB", 0.0));
    replacements.insert("%[%.2f]A%[%s]V", format!("{:.2}K/s", 7181.07));

    // Free space on the server's disk
    let session_lock = session.lock().await;
    let current_dir = session_lock.current_dir.clone();
    let path = PathBuf::from(current_dir);
    drop(session_lock);

    match get_site_free_space(&path) {
        Ok(free_space_mb) => {
            let formatted_space = format_free_space(free_space_mb as f64);
            replacements.insert("%[%.0f]FMB", formatted_space);
        }
        Err(e) => {
            error!("Failed to get free space: {:?}", e);
            replacements.insert("%[%.0f]FMB", "0MB".to_string());
        }
    }

    replacements.insert("%[%s]b", "DEFAULT".to_string());
    replacements.insert("%[%.1f]Ic%[%s]Y", format!("{:.1}MB", 14.6));
    replacements.insert("%[%s]Ir", "Unlimited".to_string());

    let statline = cleanup_cookie_statline(&statline_template, &replacements);

    let mut writer = writer.lock().await;
    writer.write_all(statline.as_bytes()).await?;
    writer.flush().await?;
    debug!("Statline sent successfully : {:?}", STATLINE_PATH);

    Ok(())
}
