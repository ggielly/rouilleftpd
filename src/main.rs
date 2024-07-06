mod config;
mod constants;
mod cookies;
mod core_cli;
mod core_ftpcommand;
mod core_log;
mod core_network;
mod helpers;
mod ipc;
mod server;
mod session;
mod watchdog;

use crate::config::Config;
use crate::core_cli::Cli;
use crate::helpers::handle_command;
use crate::helpers::load_config;

use anyhow::Result;
use colored::*;
use env_logger::{Builder, Env};
use ipc::Ipc;
use std::io::Write;
use std::sync::Arc;
use std::sync::Mutex;
use structopt::StructOpt;
use tokio;

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

    let config: Config = load_config(config_path)?;

    // Handle the Result<Ipc, _> from Ipc::new
    let ipc: Arc<Ipc> = match Ipc::new(config.server.ipc_key.clone()) {
        Ok(instance) => Arc::new(instance),
        Err(e) => {
            eprintln!("Failed to create IPC: {}", e);
            // Create a dummy IPC instance with an empty key if there's an error
            Arc::new(Ipc {
                ipc_key: String::new(),
                memory: Arc::new(Mutex::new(Vec::new())), // Empty shared memory
            })
        }
    };

    // Example usage
    let username = "rouilleftpd";
    let command = "LIST";
    let download_speed = 512.0; // Example value in KB/s
    let upload_speed = 9000.0; // Example value in KB/s

    handle_command(&ipc, username, command, download_speed, upload_speed).await;

    watchdog::start_watchdog(ipc.clone()); // Start the watchdog process

    server::run(config, ipc).await?;

    Ok(())
}
