mod core_cli;
mod core_ftpcommand;
mod core_log;
mod core_network;
mod helpers;
mod ipc;
mod server;
mod session;

use crate::core_cli::Cli;
use anyhow::{Context, Result};
use colored::*;
use env_logger::{Builder, Env};
use ipc::Ipc;
use serde::Deserialize;
use std::fs;
use std::io::Write;
use structopt::StructOpt;
use tokio;

pub mod constants;


#[derive(Debug, Deserialize)]
struct ServerConfig {
    listen_port: u16,
    pasv_address: String,
    ipc_key: String,
    chroot_dir: String,
    min_homedir: String,
}

#[derive(Debug, Deserialize)]
struct Config {
    server: ServerConfig,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Cli::from_args();

    // Initialize the logger with a custom format and colors
    Builder::from_env(Env::default().default_filter_or("info"))
        .format(|buf, record| {
            let timestamp = buf.timestamp().to_string();
            let level = match record.level() {
                log::Level::Error => record.level().to_string().red(),
                log::Level::Warn => record.level().to_string().yellow(),
                log::Level::Info => record.level().to_string().green(),
                log::Level::Debug => record.level().to_string().blue(),
                log::Level::Trace => record.level().to_string().white(),
            };
            writeln!(buf, "[{}] [{}] {}", timestamp, level, record.args())
        })
        .init();

    let default_config_path = if cfg!(target_os = "windows") {
        "C:\\src\\rouilleFTPd\\rouilleftpd\\etc\\rouilleftpd.conf"
    } else {
        "/etc/rouilleftpd.conf"
    };

    let config_path = if args.config.is_empty() {
        default_config_path
    } else {
        args.config.as_str()
    };
    let config = load_config(config_path)?;

    let ipc = Ipc::new(config.server.ipc_key.clone());

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
