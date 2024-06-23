use crate::Config;
use crate::session::Session;
use crate::core_ftpcommand::site::site_adduser::handle_adduser_command;
use log::{info, warn};
use std::sync::Arc;
use tokio::io::AsyncWriteExt;
use tokio::net::TcpStream;
use tokio::sync::Mutex;

pub async fn handle_site_command(
    writer: Arc<Mutex<TcpStream>>,
    config: Arc<Config>,
    session: Arc<Mutex<Session>>,
    arg: String,  // Take the original string argument
) -> Result<(), std::io::Error> {
    info!("Received SITE command with args: {}", arg); // Log the raw arg

    // Split args, handling potential errors (no more splitn)
    let args: Vec<String> = arg.trim().split_whitespace().map(String::from).collect();

    if args.is_empty() {
        warn!("No subcommand provided for SITE command.");
        let mut writer = writer.lock().await;
        writer.write_all(b"501 Syntax error in parameters or arguments.\r\n").await?;
        return Ok(());
    }

    let subcommand = args[0].to_lowercase();  
    let sub_args = if args.len() > 1 { args[1..].to_vec() } else { Vec::new() };

    match subcommand.as_str() { 
        "adduser" => {
            info!("Handling SITE ADDUSER command with args: {:?}", sub_args);
            handle_adduser_command(writer, config, session, sub_args).await
        }
        _ => {
            warn!("Unknown subcommand for SITE command: {}", subcommand);
            let mut writer = writer.lock().await;
            writer.write_all(b"502 Command not implemented.\r\n").await?;
            Ok(())
        }
    }
}
