use crate::core_ftpcommand::site::helper::respond_with_error;
use crate::core_ftpcommand::site::site_addip::handle_site_addip_command;
use crate::core_ftpcommand::site::site_adduser::handle_site_adduser_command;
use crate::core_ftpcommand::site::site_chmod::handle_site_chmod_command;
use crate::core_ftpcommand::site::site_delip::handle_site_delip_command;
use crate::core_ftpcommand::site::site_deluser::handle_site_deluser_command;
use crate::core_ftpcommand::site::site_group::handle_site_group_command;
use crate::core_ftpcommand::site::site_idle::handle_site_idle_command;
use crate::core_ftpcommand::site::site_new::handle_site_new_command;
use crate::core_ftpcommand::site::site_quota::handle_site_quota_command;
use crate::core_ftpcommand::site::site_ratio::handle_site_ratio_command;
use crate::core_ftpcommand::site::site_user::handle_site_user_command;
use crate::core_ftpcommand::site::site_utime::handle_site_utime_command;
use crate::core_ftpcommand::site::site_who::handle_site_who_command;

use crate::core_quota::manager::QuotaManager;
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
    quota_manager: Option<Arc<QuotaManager>>,
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
        "RATIO" => {
            info!("Handling SITE RATIO command");
            handle_site_ratio_command(writer, config, session, sub_args, quota_manager).await
        }
        "QUOTA" => {
            info!("Handling SITE QUOTA command");
            handle_site_quota_command(writer, config, session, sub_args, quota_manager).await
        }
        "GROUP" => {
            info!("Handling SITE GROUP command");
            handle_site_group_command(writer, config, session, sub_args, quota_manager).await
        }
        "CHMOD" => {
            info!("Handling SITE CHMOD command");
            handle_site_chmod_command(writer, config, session, sub_args, quota_manager).await
        }
        "WHO" => {
            info!("Handling SITE WHO command");
            handle_site_who_command(writer, config, session, sub_args, quota_manager).await
        }
        "NEW" => {
            info!("Handling SITE NEW command");
            handle_site_new_command(writer, config, session, sub_args, quota_manager).await
        }
        "IDLE" => {
            info!("Handling SITE IDLE command");
            handle_site_idle_command(writer, config, session, sub_args, quota_manager).await
        }
        _ => {
            warn!("Unknown SITE subcommand: {}", subcommand);
            respond_with_error(&writer, b"502 Command not implemented.\r\n").await?;
            Ok(())
        }
    }
}
