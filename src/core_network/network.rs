use tokio::net::{TcpListener, TcpStream};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use anyhow::{Result, Context};
use std::fs::File;
use std::io::Read;

use crate::core_ftpcommand::{user::handle_user_command, pass::handle_pass_command, quit::handle_quit_command};
use crate::core_log::logger::log_message; // Ensure this path is correct

pub async fn start_server(listen_port: u16) -> Result<()> {
    let listener = TcpListener::bind(format!("0.0.0.0:{}", listen_port)).await?;
    println!("Server listening on port {}", listen_port);

    loop {
        let (socket, addr) = listener.accept().await?;
        println!("New connection from {:?}", addr);

        tokio::spawn(handle_connection(socket));
    }
}

async fn handle_connection(mut socket: TcpStream) -> Result<()> {
    // Load and send the banner text
    let banner_path = if cfg!(target_os = "windows") {
        "C:\\src\\rouilleFTPd\\rouilleftpd\\ftp-data\\text\\banner.txt"
    } else {
        "/ftp-data/text/banner.txt"
    };

    let banner_text = load_banner(banner_path)?;

    // Send the banner text with FTP status code 220
    socket.write_all(format!("220-{}\r\n", banner_text).as_bytes()).await?;

    let mut reader = BufReader::new(socket);
    let mut buffer = String::new();

    loop {
        buffer.clear();
        let n = reader.read_line(&mut buffer).await?;

        if n == 0 {
            break;
        }

        let command = buffer.trim();
        log_message(&format!("Received command: {}", command));
        
        let parts: Vec<&str> = command.splitn(2, ' ').collect();
        let cmd = parts[0].to_uppercase();
        let arg = parts.get(1).map(|s| s.trim()).unwrap_or("");

        match cmd.as_str() {
            "USER" => handle_user_command(&mut reader.get_mut(), arg).await?,
            "PASS" => handle_pass_command(&mut reader.get_mut(), arg).await?,
            "QUIT" => {
                handle_quit_command(&mut reader.get_mut()).await?;
                break;
            }
            _ => {
                reader.get_mut().write_all(b"502 Command not implemented.\r\n").await?;
            }
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
