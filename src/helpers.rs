//use crate::tokio::fs;
use crate::{Config, Ipc};
use anyhow::{Context, Result};
use log::error;
use log::info;
use std::fs;
use std::fs::File;
use std::io::Read;
use std::sync::Arc;
use tokio::io::AsyncWriteExt;
use tokio::net::TcpStream;
use tokio::sync::Mutex;

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

// Alpha version
pub fn update_user_record(
    ipc: &Ipc,
    username: &str,
    command: &str,
    download_speed: f32,
    upload_speed: f32,
) {
    let mut username_bytes = [0u8; 32];
    let mut command_bytes = [0u8; 32];

    // Copy the username and command into the fixed-size arrays
    username_bytes[..username.len()].copy_from_slice(username.as_bytes());
    command_bytes[..command.len()].copy_from_slice(command.as_bytes());

    // Create a new UserRecord instance with the provided data
    let record = UserRecord {
        username: username_bytes,
        command: command_bytes,
        download_speed,
        upload_speed,
    };

    // Debug output before writing
    eprintln!("Creating user record: {:?}", record);

    // Write the UserRecord instance to the shared memory
    ipc.write_user_record(record);

    // Ensure the write operation
    eprintln!("User record updated: {:?}", record);
    eprintln!("Shared memory: {:?}", ipc.memory);
}

// Example function that handles a command
pub async fn handle_command(
    ipc: &Ipc,
    username: &str,
    command: &str,
    download_speed: f32,
    upload_speed: f32,
) {
    // Your existing command handling logic

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
        return Err(anyhow::Error::msg("Banner file is empty"));
    }

    info!("Banner file loaded successfully: {}", path);
    Ok(config_str)
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct UserRecord {
    pub username: [u8; 32],  // Fixed-size array for username (32 bytes)
    pub command: [u8; 32],   // Fixed-size array for command (32 bytes)
    pub download_speed: f32, // Download speed
    pub upload_speed: f32,   // Upload speed
}

impl UserRecord {
    pub fn from_bytes(bytes: &[u8]) -> Self {
        let username = {
            let mut array = [0u8; 32];
            array.copy_from_slice(&bytes[0..32]);
            array
        };
        let command = {
            let mut array = [0u8; 32];
            array.copy_from_slice(&bytes[32..64]);
            array
        };
        let download_speed = f32::from_ne_bytes([bytes[64], bytes[65], bytes[66], bytes[67]]);
        let upload_speed = f32::from_ne_bytes([bytes[68], bytes[69], bytes[70], bytes[71]]);

        UserRecord {
            username,
            command,
            download_speed,
            upload_speed,
        }
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&self.username);
        bytes.extend_from_slice(&self.command);
        bytes.extend_from_slice(&self.download_speed.to_ne_bytes());
        bytes.extend_from_slice(&self.upload_speed.to_ne_bytes());
        bytes
    }
}
