use crate::core_ftpcommand::site::helper::respond_with_error;
use crate::core_ftpcommand::site::site_addip::handle_site_addip_command;
use crate::core_ftpcommand::site::site_adduser::handle_site_adduser_command;
use crate::core_ftpcommand::site::site_delip::handle_site_delip_command;
use crate::core_ftpcommand::site::site_deluser::handle_site_deluser_command;
use crate::core_ftpcommand::site::site_user::handle_site_user_command;
use crate::core_ftpcommand::site::site_utime::handle_site_utime_command;


use crate::{session::Session, Config}; // for Config and Session
use log::{info, warn}; // for logging
use std::sync::Arc; // for Arc
use tokio::net::TcpStream; // for TcpStream
use tokio::sync::Mutex; // for Mutex

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
            handle_site_adduser_command(writer, config, session, sub_args).await
        }
        "ADDIP" => {
            info!("Handling SITE ADDIP command with args: {:?}", sub_args);
            handle_site_addip_command(writer, config, session, sub_args).await
        }
        "DELIP" => {
            info!("Handling SITE DELIP command with args: {:?}", sub_args);
            handle_site_delip_command(writer, config, session, sub_args).await
        }
        "DELUSER" => {
            info!("Handling SITE DELUSER command with args: {:?}", sub_args);
            handle_site_deluser_command(writer, config, session, sub_args).await
        }
        "USER" => {
            if sub_args.len() == 1 {
                info!("Handling SITE USER command for user: {:?}", sub_args[0]);
                handle_site_user_command(writer, config, session, sub_args[0].clone()).await
            } else {
                warn!("Invalid arguments for SITE USER command: {:?}", sub_args);
                respond_with_error(&writer, b"501 Syntax error in parameters or arguments.\r\n")
                    .await
            }
        }
        "UTIME" => {
            info!("Handling SITE UTIME command with args: {:?}", sub_args);
            handle_site_utime_command(writer, config, session, sub_args.join(" ")).await
        }
        _ => {
            warn!("Unknown SITE subcommand: {}", subcommand);
            respond_with_error(&writer, b"502 Command not implemented.\r\n").await?;
            Ok(())
        }
    }
}
