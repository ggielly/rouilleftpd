use crate::{session::Session, Config};
use anyhow::Result;
use log::{error, info};
use std::net::IpAddr;
use std::sync::Arc;
use tokio::io::AsyncWriteExt;
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::Mutex;

/// Sets up a passive mode (PASV) listener and sends the response to the client.
pub async fn handle_pasv_command(
    writer: Arc<Mutex<TcpStream>>,
    config: Arc<Config>,
    session: Arc<Mutex<Session>>,
    _arg: String,
) -> Result<(), std::io::Error> {
    let pasv_ip: IpAddr = config
        .server
        .pasv_address
        .parse()
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidInput, e))?;

    // Set up the passive mode listener
    let (listener, pasv_response) = setup_pasv_listener(pasv_ip).await?;

    // Send PASV response to the client
    {
        let mut writer = writer.lock().await;
        writer.write_all(pasv_response.as_bytes()).await?;
    }

    // Clone writer and session to move into the spawned task
    let writer_clone = Arc::clone(&writer);
    let session_clone = Arc::clone(&session);

    // Accept the incoming connection in a separate task
    tokio::spawn(async move {
        match accept_pasv_connection(listener).await {
            Ok(data_stream) => {
                let mut session = session_clone.lock().await;
                session.data_stream = Some(Arc::new(Mutex::new(data_stream)));
                info!("PASV connection established");
            }
            Err(e) => {
                error!("Failed to accept data connection: {}", e);
                let mut writer = writer_clone.lock().await;
                writer
                    .write_all(b"425 Can't open data connection.\r\n")
                    .await
                    .ok(); // Ignore write error
            }
        }
    });

    Ok(())
}

/// Sets up a passive mode (PASV) listener.
/// Returns the listener and the formatted PASV response.
pub async fn setup_pasv_listener(pasv_ip: IpAddr) -> Result<(TcpListener, String), std::io::Error> {
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
pub async fn accept_pasv_connection(listener: TcpListener) -> Result<TcpStream, std::io::Error> {
    let (data_stream, _) = listener.accept().await?;
    Ok(data_stream)
}
