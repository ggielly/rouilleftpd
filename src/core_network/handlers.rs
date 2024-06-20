use std::sync::Arc;
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::Mutex;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::io::AsyncWriteExt;
use crate::{Config, Session, initialize_command_handlers};

async fn handle_connection(stream: TcpStream, config: Arc<Config>) -> Result<(), std::io::Error> {
    let handlers = initialize_command_handlers();
    let session = Arc::new(Mutex::new(Session::new()));

    let reader = BufReader::new(&stream);
    let writer = Arc::new(Mutex::new(stream));

    let mut lines = reader.lines();

    while let Some(Ok(line)) = lines.next_line().await {
        let parts: Vec<&str> = line.splitn(2, ' ').collect();
        let command_name = parts[0].to_uppercase();
        let command_arg = if parts.len() > 1 { parts[1].to_string() } else { String::new() };

        if let Some(handler) = handlers.get(&command_name) {
            handler(writer.clone(), config.clone(), session.clone(), command_arg).await?;
        } else {
            // Handle unknown command
            let mut writer = writer.lock().await;
            writer.write_all(b"502 Command not implemented.\r\n").await?;
        }
    }

    Ok(())
}