use tokio::net::{TcpListener, TcpStream};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use anyhow::{Result, Context};
use std::fs::File;
use std::io::Read;
use std::sync::Arc;
use tokio::sync::Mutex;
use crate::Config;
use crate::ipc::Ipc;
use crate::core_log::logger::log_message;
use crate::core_ftpcommand::handlers::initialize_command_handlers;

pub async fn start_server(listen_port: u16, config: Arc<Config>, ipc: Ipc) -> Result<()> {
    let listener = TcpListener::bind(format!("0.0.0.0:{}", listen_port)).await?;
    log_message(&format!("Server listening on port {}", listen_port));

    loop {
        let (socket, addr) = listener.accept().await?;
        log_message(&format!("New connection from {:?}", addr));

        let config = Arc::clone(&config);
        let ipc = ipc.clone();

        tokio::spawn(async move {
            if let Err(e) = handle_connection(socket, config, ipc).await {
                log_message(&format!("Connection error: {:?}", e));
            }
            log_message(&format!("Connection closed for {:?}", addr));
        });
    }
}

pub async fn handle_connection(socket: TcpStream, config: Arc<Config>, _ipc: crate::ipc::Ipc) -> Result<()> {
    // Load and send the banner text
    let banner_path = if cfg!(target_os = "windows") {
        "ftp-data/text/banner.txt"
    } else {
        "ftp-data/text/banner.txt"
    };

    let banner_text = load_banner(banner_path)?;

    let socket = Arc::new(Mutex::new(socket));
    {
        let mut socket = socket.lock().await;
        socket.write_all(format!("220-{}\r\n", banner_text).as_bytes()).await?;
    }

    let handlers = initialize_command_handlers();
    let session = Arc::new(Mutex::new(Session::new()));
    let mut buffer = String::new();

    loop {
        buffer.clear();
        {
            let mut locked_socket = socket.lock().await;
            let mut reader = BufReader::new(&mut *locked_socket);
            let n = reader.read_line(&mut buffer).await?;
            drop(locked_socket);

            if n == 0 {
                break;
            }
        }

        let command = buffer.trim();
        log_message(&format!("Received command: {}", command));

        let parts: Vec<&str> = command.split_whitespace().collect();
        let cmd = parts.get(0).map(|s| s.to_ascii_uppercase()).unwrap_or_default();
        let arg = parts.get(1).map(|s| s.to_string()).unwrap_or_default();

        if let Some(handler) = handlers.get(&cmd) {
            handler(Arc::clone(&socket), Arc::clone(&config), Arc::clone(&session), arg).await?;
        } else {
            let mut socket = socket.lock().await;
            socket.write_all(b"502 Command not implemented.\r\n").await?;
        }
    }
    Ok(())
}

fn load_banner(path: &str) -> Result<String> {
    let mut file = File::open(path)
        .with_context(|| format!("Failed to open banner file: {}", path))?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)
        .with_context(|| format!("Failed to read banner file: {}", path))?;
    Ok(contents)
}
