use crate::core_network::network;
use crate::ipc::Ipc;
use crate::Config;
use anyhow::Result;
use log::{error, info};
use std::sync::Arc;

use crate::session::Session;
use std::path::PathBuf;

/// Runs the FTP server with the provided configuration and IPC key.
///
/// This function initializes the server configuration and starts the FTP server,
/// logging significant steps and potential issues.
///
/// # Arguments
///
/// * `config` - The server configuration.
/// * `ipc` - The IPC instance for inter-process communication.
///
/// # Returns
///
/// Result<(), anyhow::Error> indicating the success or failure of the operation.
pub async fn run(config: Config, ipc: Arc<Ipc>) -> Result<()> {
    info!("Starting server with config: {:?}", config);
    info!("IPC Key: {:?}", ipc.ipc_key);

    // Start the FTP server
    match network::start_server(config.server.listen_port, Arc::new(config), ipc).await {
        Ok(_) => info!("Server started successfully."),
        Err(e) => {
            error!("Failed to start server: {}", e);
            return Err(e);
        }
    }

    Ok(())
}

pub fn initialize_session(config: &Config) -> Session {
    let base_path = PathBuf::from(&config.server.chroot_dir)
        .join(config.server.min_homedir.trim_start_matches('/'))
        .canonicalize()
        .unwrap();

    Session::new(base_path)
}
