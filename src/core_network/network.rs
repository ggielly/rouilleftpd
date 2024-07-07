use crate::core_ftpcommand::ftpcommand::FtpCommand;
use crate::core_ftpcommand::handlers::initialize_command_handlers;
use crate::core_log::logger::log_message;
use crate::helpers::load_banner;
use crate::ipc::update_ipc;
use crate::session::Session;
use crate::Config;
use crate::Ipc;
use anyhow::Result;
use log::{error, info};

use std::path::PathBuf;
use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::Mutex;

use crate::helpers::send_response;

use crate::core_network::pasv::accept_pasv_connection;
use crate::core_network::pasv::setup_pasv_listener;

pub async fn start_server(port: u16, config: Arc<Config>, ipc: Arc<Ipc>) -> Result<()> {
    let listener = TcpListener::bind(format!("0.0.0.0:{}", port)).await?;
    info!("Server listening on port {}", port);

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
                error!("Connection error: {:?}", e);
            }
            info!("Connection closed for {:?}", addr);
        });
    }
}

pub async fn handle_connection(
    socket: TcpStream,
    config: Arc<Config>,
    session: Arc<Mutex<Session>>,
    ipc: Arc<Ipc>,
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
            "SIZE" => FtpCommand::SIZE,
            _ => {
                let mut socket = socket.lock().await;
                socket
                    .write_all(b"502 Command not implemented.\r\n")
                    .await?;
                continue;
            }
        };

        let args = parts[1..].to_vec();

        // Update IPC with the command and username
        let username = {
            let session = session.lock().await;
            session.username.clone().unwrap_or_default()
        };
        update_ipc(Arc::clone(&ipc), &username, &cmd_str, 0.0, 0.0);

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
