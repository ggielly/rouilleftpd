use crate::core_ftpcommand::site::helper::respond_with_error;
use crate::core_ftpcommand::site::site_addip::handle_addip_command;
use crate::core_ftpcommand::site::site_adduser::handle_adduser_command;
use crate::{session::Session, Config};
use log::{info, warn};
use std::sync::Arc;
use tokio::net::TcpStream;
use tokio::sync::Mutex;

pub async fn handle_site_command(
    writer: Arc<Mutex<TcpStream>>,
    config: Arc<Config>,
    session: Arc<Mutex<Session>>,
    arg: String,
) -> Result<(), std::io::Error> {
    let mut args: Vec<&str> = arg.trim().split(' ').collect();

    if args.is_empty() {
        warn!("No subcommand provided for SITE command.");
        respond_with_error(&writer, b"501 Syntax error in parameters or arguments.\r\n").await?;
        return Ok(());
    }

    let subcommand = args.remove(0).to_ascii_uppercase();
    let sub_args = args.iter().map(|s| s.to_string()).collect();

    match subcommand.as_str() {
        "ADDUSER" => {
            info!("Handling SITE ADDUSER command with args: {:?}", sub_args);
            handle_adduser_command(writer, config, session, sub_args).await
        }
        "ADDIP" => {
            info!("Handling SITE ADDIP command with args: {:?}", sub_args);
            handle_addip_command(writer, config, session, sub_args).await
        }
        _ => {
            warn!("Unknown SITE subcommand: {}", subcommand);
            respond_with_error(&writer, b"502 Command not implemented.\r\n").await?;
            Ok(())
        }
    }
}
