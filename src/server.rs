use crate::core_network::network;
use crate::ipc::Ipc;
use crate::Config;
use anyhow::Result;
use std::sync::Arc;

pub async fn run(config: Config, ipc: Ipc) -> Result<()> {
    println!("Starting server with config: {:?}", config);
    println!("IPC Key: {:?}", ipc.ipc_key);

    // Start the FTP server
    network::start_server(config.server.listen_port, Arc::new(config), ipc).await?;

    Ok(())
}
