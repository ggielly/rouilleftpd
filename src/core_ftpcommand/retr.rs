use crate::core_network::handlers::handle_fxp_transfer;
use crate::helpers::{sanitize_input, send_response};
use crate::{session::Session, Config};
use log::{error, info, warn};
use std::io::{self, ErrorKind};
use std::sync::Arc;
use tokio::{
    fs::File,
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
    sync::Mutex,
};

pub async fn handle_retr_command(
    writer: Arc<Mutex<TcpStream>>,
    config: Arc<Config>,
    session: Arc<Mutex<Session>>,
    arg: String,
) -> io::Result<()> {
    if arg.trim().is_empty() {
        warn!("RETR command received with no arguments");
        send_response(&writer, b"501 Syntax error in parameters or arguments.\r\n").await?;
        return Ok(());
    }

    let sanitized_arg = sanitize_input(&arg);
    info!("Received RETR command with argument: {}", sanitized_arg);

    let resolved_path = {
        let session = session.lock().await;
        let base_path = session.base_path.clone();
        let file_path = base_path.join(&sanitized_arg);
        file_path.canonicalize().unwrap_or_else(|_| file_path)
    };

    if !resolved_path.starts_with(&session.lock().await.base_path) {
        error!("Path is outside of the allowed area: {:?}", resolved_path);
        send_response(&writer, b"550 Path is outside of the allowed area.\r\n").await?;
        return Ok(());
    }

    let mut file = File::open(&resolved_path)
        .await
        .map_err(|e| io::Error::new(ErrorKind::Other, format!("Failed to open file: {:?}", e)))?;

    send_response(
        &writer,
        b"150 File status okay; about to open data connection.\r\n",
    )
    .await?;

    let data_stream = {
        let session = session.lock().await;
        session.data_stream.clone()
    };

    if let Some(data_stream) = data_stream {
        let mut data_stream = data_stream.lock().await;

        let buffer_size = config.server.download_buffer_size.unwrap_or(128 * 1024);
        let mut buffer = vec![0; buffer_size]; // Use configured buffer size

        while let Ok(bytes_read) = file.read(&mut buffer).await {
            if bytes_read == 0 {
                break; // End of file
            }

            data_stream
                .write_all(&buffer[..bytes_read])
                .await
                .map_err(|e| {
                    io::Error::new(
                        ErrorKind::Other,
                        format!("Error writing to data stream: {:?}", e),
                    )
                })?;
        }

        send_response(&writer, b"226 Transfer complete.\r\n").await?;
        info!("File transferred successfully: {:?}", resolved_path);
    } else {
        error!("Data connection is not established.");
        send_response(&writer, b"425 Can't open data connection.\r\n").await?;
    }

    Ok(())
}
