use tokio::net::{TcpListener, TcpStream};
use anyhow::Result;
use std::sync::Arc;
use tokio::sync::Mutex;
use crate::Config;
use crate::core_network::Session;
use log::{info, error};
use tokio::io::AsyncWriteExt;
use std::net::IpAddr;

/// Sets up a passive mode (PASV) listener.
/// Returns the listener and the formatted PASV response.
pub async fn setup_pasv_listener(pasv_ip: IpAddr) -> Result<(TcpListener, String)> {
    let listener = TcpListener::bind((pasv_ip, 0)).await?;
    let addr = listener.local_addr()?;

    let ip_string = pasv_ip.to_string();
    let ip_parts: Vec<&str> = ip_string.split('.').collect();
    let pasv_response = format!(
        "227 Entering Passive Mode ({},{},{},{},{},{}).\r\n",
        ip_parts[0],
        ip_parts[1],
        ip_parts[2],
        ip_parts[3],
        addr.port() / 256,
        addr.port() % 256
    );
    Ok((listener, pasv_response))
}

/// Accepts the incoming connection on the passive listener.
pub async fn accept_pasv_connection(listener: TcpListener) -> Result<TcpStream> {
    let (data_stream, _) = listener.accept().await?;
    Ok(data_stream)
}

/// Handles the PASV (Passive Mode) FTP command.
///
/// This function sets up a passive mode listener and waits for a connection from the client.
/// It sends the appropriate response back to the client with the IP and port information.
///
/// # Arguments
///
/// * `writer` - A shared, locked TCP stream for writing responses to the client.
/// * `config` - A shared server configuration.
/// * `session` - A shared, locked session containing the user's current state.
/// * `arg` - The argument provided by the client (not used for PASV).
///
/// # Returns
///
/// Result<(), std::io::Error> indicating the success or failure of the operation.
pub async fn handle_pasv_command(
    writer: Arc<Mutex<TcpStream>>,
    config: Arc<Config>,
    session: Arc<Mutex<Session>>,
    _arg: String,
) -> Result<(), std::io::Error> {
    let pasv_ip: IpAddr = config.server.pasv_address.parse().map_err(|e| {
        std::io::Error::new(std::io::ErrorKind::InvalidInput, e)
    })?;

    // Set up the passive mode listener
    match setup_pasv_listener(pasv_ip).await {
        Ok((listener, pasv_response)) => {
            // Send PASV response to the client
            let mut writer = writer.lock().await;
            writer.write_all(pasv_response.as_bytes()).await?;

            // Accept the incoming connection
            match accept_pasv_connection(listener).await {
                Ok(data_stream) => {
                    let mut session = session.lock().await;
                    session.data_stream = Some(Arc::new(Mutex::new(data_stream)));
                }
                Err(e) => {
                    error!("Failed to accept data connection: {}", e);
                    writer.write_all(b"425 Can't open data connection.\r\n").await?;
                }
            }
        }
        Err(e) => {
            error!("Failed to set up passive listener: {}", e);
            let mut writer = writer.lock().await;
            writer.write_all(b"425 Can't open data connection.\r\n").await?;
        }
    }

    Ok(())
}
