use crate::core_ftpcommand::handlers::initialize_command_handlers;
use crate::session::Session;
use crate::Config;
use std::sync::Arc;
use tokio::io::AsyncWriteExt;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::net::TcpStream;
use tokio::sync::Mutex;

use std::path::PathBuf;
use tokio::io::AsyncReadExt;

pub async fn handle_fxp_transfer(
    source_session: Arc<Mutex<Session>>,
    dest_session: Arc<Mutex<Session>>,
) -> Result<(), std::io::Error> {
    let source_data_stream = {
        let session = source_session.lock().await;
        session.data_stream.clone()
    };

    let dest_data_stream = {
        let session = dest_session.lock().await;
        session.data_stream.clone()
    };

    if let (Some(source), Some(dest)) = (source_data_stream, dest_data_stream) {
        let mut source = source.lock().await;
        let mut dest = dest.lock().await;

        let mut buffer = vec![0; 8192];
        loop {
            let bytes_read = source.read(&mut buffer).await?;
            if bytes_read == 0 {
                break;
            }
            dest.write_all(&buffer[..bytes_read]).await?;
        }
    } else {
        return Err(std::io::Error::new(
            std::io::ErrorKind::NotConnected,
            "One or both data streams are not connected",
        ));
    }

    Ok(())
}
