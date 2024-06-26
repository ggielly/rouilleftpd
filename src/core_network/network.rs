use crate::core_ftpcommand::ftpcommand::FtpCommand;
use crate::core_ftpcommand::handlers::initialize_command_handlers;
use crate::core_log::logger::log_message;
use crate::session::Session;
use crate::Config;
use anyhow::{Context, Result};
use log::{error, info};
use std::fs::File;
use std::io::Read;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::Mutex;

use crate::core_ftpcommand::utils::send_response;

use crate::core_network::pasv::accept_pasv_connection;
use crate::core_network::pasv::setup_pasv_listener;

use std::net::SocketAddr;

pub async fn start_server(
    listen_port: u16,
    config: Arc<Config>,
    ipc: crate::ipc::Ipc,
) -> Result<()> {
    let listener = TcpListener::bind(format!("0.0.0.0:{}", listen_port)).await?;
    //log_message(&format!("Server listening on port {}", listen_port));
    info!("Server listening on port {}", listen_port);

    let base_path = PathBuf::from(&config.server.chroot_dir)
        .join(config.server.min_homedir.trim_start_matches('/'))
        .canonicalize()
        .unwrap();

    loop {
        let (socket, addr) = listener.accept().await?;
        info!("New connection from {:?}", addr);

        let config = Arc::clone(&config);
        let session = Arc::new(Mutex::new(Session::new(base_path.clone())));
        let ipc_clone = ipc.clone();

        tokio::spawn(async move {
            if let Err(e) = handle_connection(socket, config, session, ipc_clone).await {
                log_message(&format!("Connection error: {:?}", e));
            }
            log_message(&format!("Connection closed for {:?}", addr));
        });
    }
}

pub async fn handle_connection(
    socket: TcpStream,
    config: Arc<Config>,
    session: Arc<Mutex<Session>>,
    _ipc: crate::ipc::Ipc,
) -> Result<()> {
    let banner_path = if cfg!(target_os = "windows") {
        "ftp-data/text/banner.txt"
    } else {
        "ftp-data/text/banner.txt"
    };

    let banner_text = load_banner(banner_path)?;

    let socket = Arc::new(Mutex::new(socket));
    {
        let mut socket = socket.lock().await;
        socket
            .write_all(format!("220-{}\r\n", banner_text).as_bytes())
            .await?;
        socket.write_all(b"220 This is a banner !#%.\r\n").await?;
    }

    let handlers = initialize_command_handlers();
    let mut buffer = String::new();
    let mut data_stream: Option<Arc<Mutex<TcpStream>>> = None;

    loop {
        buffer.clear();
        {
            let mut locked_socket = socket.lock().await;
            let mut reader = BufReader::new(&mut *locked_socket);
            let n = reader.read_line(&mut buffer).await?;
            drop(locked_socket);

            if n == 0 {
                error!("Client disconnected unexpectedly");
                break;
            }
        }

        let command = buffer.trim();
        info!("Received command: {}", command);

        //log_message(&format!("Received command: {}", command));

        let parts: Vec<String> = command.split_whitespace().map(String::from).collect();

        if parts.is_empty() {
            info!("Empty command received, ignoring.");
            continue;
        }

        let cmd_str = parts[0].to_ascii_uppercase();
        let cmd = match cmd_str.as_str() {
            "ALLO" => FtpCommand::ALLO,
            "CDUP" => FtpCommand::CDUP,
            "FEAT" => FtpCommand::FEAT,
            "SYST" => FtpCommand::SYST,
            "SITE" => FtpCommand::SITE,
            "STOR" => FtpCommand::STOR,
            "USER" => FtpCommand::USER,
            "PASS" => FtpCommand::PASS,
            "QUIT" => FtpCommand::QUIT,
            "PWD" => FtpCommand::PWD,
            "LIST" => FtpCommand::LIST,
            "CWD" => FtpCommand::CWD,
            "NOOP" => FtpCommand::NOOP,
            "MKD" => FtpCommand::MKD,
            "RMD" => FtpCommand::RMD,
            "DELE" => FtpCommand::DELE,
            "RNFR" => FtpCommand::RNFR,
            "RNTO" => FtpCommand::RNTO,
            "RETR" => FtpCommand::RETR,
            "PORT" => FtpCommand::PORT,
            "PASV" => FtpCommand::PASV,
            "TYPE" => FtpCommand::TYPE,
            _ => {
                let mut socket = socket.lock().await;
                socket
                    .write_all(b"502 Command not implemented.\r\n")
                    .await?;
                continue;
            }
        };

        let args = parts[1..].to_vec();

        if let Some(handler) = handlers.get(&cmd) {
            if cmd == FtpCommand::PASV {
                let (listener, pasv_response) =
                    setup_pasv_listener(config.server.pasv_address.parse().unwrap()).await?;
                let mut writer = socket.lock().await;
                writer.write_all(pasv_response.as_bytes()).await?;

                // Accept the connection asynchronously
                let socket_clone = Arc::clone(&socket);
                let session_clone = Arc::clone(&session);
                tokio::spawn(async move {
                    match accept_pasv_connection(listener).await {
                        Ok(stream) => {
                            let mut session = session_clone.lock().await;
                            session.data_stream = Some(Arc::new(Mutex::new(stream)));
                        }
                        Err(err) => {
                            error!("Failed to accept PASV connection: {:?}", err);
                            send_response(&socket_clone, b"425 Cannot open data connection.\r\n")
                                .await
                                .ok();
                        }
                    }
                });
                continue; // Move to the next iteration since PASV is handled separately
            }

            let handler_result = if cmd == FtpCommand::STOR || cmd == FtpCommand::RETR {
                // Execute handler with data_stream
                handler(
                    Arc::clone(&socket),
                    Arc::clone(&config),
                    Arc::clone(&session),
                    args.join(" "),
                    data_stream.clone(),
                )
                .await
            } else {
                // Execute handler without data_stream
                handler(
                    Arc::clone(&socket),
                    Arc::clone(&config),
                    Arc::clone(&session),
                    args.join(" "),
                    None,
                )
                .await
            };

            // Check for handler errors and reset data_stream
            if let Err(e) = handler_result {
                log_message(&format!("Error handling command {}: {:?}", cmd_str, e));
                data_stream = None; // Reset data_stream after use
            }
        }
    }
    Ok(())
}

/*
async fn setup_pasv_listener(address: SocketAddr) -> Result<(TcpListener, String)> {
    let listener = TcpListener::bind(address).await?;
    let local_addr = listener.local_addr()?;
    let ip_string = local_addr.ip().to_string(); // Store the string in a variable
    let ip_parts: Vec<_> = ip_string.split('.').collect();
    let pasv_response = format!(
        "227 Entering Passive Mode ({},{},{},{},{},{}).\r\n",
        ip_parts[0], ip_parts[1], ip_parts[2], ip_parts[3],
        (local_addr.port() / 256),
        (local_addr.port() % 256)
    );
    Ok((listener, pasv_response))
}


async fn accept_pasv_connection(listener: TcpListener) -> Result<TcpStream> {
    let (stream, _) = listener.accept().await?;
    Ok(stream)
}
*/

fn load_banner(path: &str) -> Result<String> {
    let mut file = File::open(path).context(format!("Failed to open banner file: {}", path))?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)
        .context(format!("Failed to read banner file: {}", path))?;
    Ok(contents)
}
