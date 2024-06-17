use crate::{Config, ipc::Ipc};
use anyhow::Result;
use crate::core_network::network;

pub async fn run(config: Config, ipc: Ipc) -> Result<()> {
    println!("Starting server with config: {:?}", config);
    println!("IPC Key: {:?}", ipc.ipc_key);

    // Start the FTP server
    network::start_server(config.server.listen_port).await?;

    Ok(())
}
