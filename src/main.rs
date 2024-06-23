mod core_cli;
mod core_ftpcommand;
mod core_log;
mod core_network;
mod ipc;
mod server;

use crate::core_cli::Cli;
use anyhow::{Context, Result};
use env_logger::{Builder, Env};
use ipc::Ipc;
use serde::Deserialize;
use std::fs;
use std::io::Write;
use structopt::StructOpt;
use tokio;

#[derive(Debug, Deserialize)]
struct ServerConfig {
    listen_port: u16,
    ipc_key: String,
    chroot_dir: String,
    min_homedir: String,
    pasv_address: String, // Add the public IP address for PASV mode
}

#[derive(Debug, Deserialize)]
struct Config {
    server: ServerConfig,
}

#[tokio::main]
async fn main() -> Result<()> {
    // Parse CLI arguments
    let args = Cli::from_args();

    // Initialize the logger with a custom format
    Builder::from_env(Env::default().default_filter_or("info"))
        .format(|buf, record| {
            let timestamp = buf.timestamp();
            writeln!(
                buf,
                "[{}] [{}] {}",
                timestamp,
                record.level(),
                record.args()
            )
        })
        .init();

    // Determine the default config path based on the OS
    let default_config_path = if cfg!(target_os = "windows") {
        "C:\\src\\rouilleFTPd\\rouilleftpd\\etc\\rouilleftpd.conf"
    } else {
        "/etc/rouilleftpd.conf"
    };

    // Load configuration from the TOML file
    let config_path = if args.config.is_empty() {
        default_config_path
    } else {
        args.config.as_str()
    };
    let mut config = load_config(config_path)?;

    // Override IPC key from CLI if provided
    if let Some(ipc_key) = args.ipc_key {
        config.server.ipc_key = ipc_key;
    }

    // Initialize IPC
    let ipc = Ipc::new(config.server.ipc_key.clone());

    // Run the FTP server
    server::run(config, ipc).await?;

    Ok(())
}

fn load_config(path: &str) -> Result<Config> {
    let config_str = fs::read_to_string(path)
        .with_context(|| format!("Failed to read configuration file: {}", path))?;
    let config = toml::from_str(&config_str)
        .with_context(|| format!("Failed to parse configuration file: {}", path))?;
    Ok(config)
}
