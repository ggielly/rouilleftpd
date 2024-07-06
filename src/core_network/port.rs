use crate::session::Session;
use crate::Config;
use anyhow::Result;
use log::{error, info};
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::io::AsyncWriteExt;
use tokio::net::TcpStream;
use tokio::sync::Mutex;

/// Sets up an active mode (PORT) connection.
/// Parses the IP address and port, validates them, and attempts to connect.
pub async fn setup_port_connection(ip: &str, port: u16) -> Result<TcpStream> {
    let addr = SocketAddr::new(ip.parse()?, port);
    let data_stream = TcpStream::connect(addr).await?;
    Ok(data_stream)
}

/// Handles the PORT (Active Mode) FTP command.
pub async fn handle_port_command(
    writer: Arc<Mutex<TcpStream>>,
    _config: Arc<Config>,
    session: Arc<Mutex<Session>>,
    arg: String,
) -> Result<(), std::io::Error> {
    // Parse the IP address and port from the argument
    let parts: Vec<&str> = arg.split(',').collect();
    if parts.len() != 6 {
        // Invalid argument format
        let mut writer = writer.lock().await;
        writer
            .write_all(b"501 Syntax error in parameters or arguments.\r\n")
            .await?;
        return Ok(());
    }

    // Validate and construct the IP address
    let ip_parts: Result<Vec<u8>, _> = parts[0..4].iter().map(|x| x.parse::<u8>()).collect();
    if ip_parts.is_err() {
        let mut writer = writer.lock().await;
        writer.write_all(b"501 Invalid IP address.\r\n").await?;
        return Ok(());
    }
    let ip_parts = ip_parts.unwrap();
    let ip = format!(
        "{}.{}.{}.{}",
        ip_parts[0], ip_parts[1], ip_parts[2], ip_parts[3]
    );

    // Validate and construct the port number
    let port_parts: Result<Vec<u8>, _> = parts[4..6].iter().map(|x| x.parse::<u8>()).collect();
    if port_parts.is_err() {
        let mut writer = writer.lock().await;
        writer.write_all(b"501 Invalid port number.\r\n").await?;
        return Ok(());
    }
    let port_parts = port_parts.unwrap();
    let port = (port_parts[0] as u16) << 8 | port_parts[1] as u16;

    info!("Received PORT command with IP: {} and port: {}", ip, port);

    // Attempt to connect to the specified address
    match setup_port_connection(&ip, port).await {
        Ok(data_stream) => {
            info!("Connection established with {}:{}", ip, port);
            let mut session = session.lock().await;
            session.data_stream = Some(Arc::new(Mutex::new(data_stream)));
            let mut writer = writer.lock().await;
            writer.write_all(b"200 Command okay.\r\n").await?;
        }
        Err(e) => {
            error!("Failed to connect to client {}: {}: {}", ip, port, e);
            let mut writer = writer.lock().await;
            writer
                .write_all(b"425 Can't open data connection.\r\n")
                .await?;
        }
    }

    Ok(())
}
