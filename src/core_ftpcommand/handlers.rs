use crate::core_ftpcommand::ftpcommand::FtpCommand;
use crate::session::Session;
use crate::Config;
use anyhow::Result;
use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use tokio::net::TcpStream;
use tokio::sync::Mutex as TokioMutex;

// Specific crates for PORT and PASV commands
use crate::core_network::pasv;
use crate::core_network::port;

type CommandHandler = Box<
    dyn Fn(
            Arc<TokioMutex<TcpStream>>,
            Arc<Config>,
            Arc<TokioMutex<Session>>,
            String,                             // Full command string
            Option<Arc<TokioMutex<TcpStream>>>, // Optional data stream
        ) -> Pin<Box<dyn Future<Output = Result<(), std::io::Error>> + Send>>
        + Send
        + Sync,
>;

pub fn initialize_command_handlers() -> HashMap<FtpCommand, Arc<CommandHandler>> {
    let mut handlers: HashMap<FtpCommand, Arc<CommandHandler>> = HashMap::new();

    handlers.insert(
        FtpCommand::MDTM,
        Arc::new(Box::new(|writer, config, session, arg, _data_stream| {
            Box::pin(crate::core_ftpcommand::mdtm::handle_mdtm_command(
                writer, config, session, arg,
            ))
        })),
    );

    handlers.insert(
        FtpCommand::SIZE,
        Arc::new(Box::new(|writer, config, session, arg, data_stream| {
            Box::pin(crate::core_ftpcommand::size::handle_size_command(
                writer,
                config,
                session,
                arg,
                data_stream,
            ))
        })),
    );

    handlers.insert(
        FtpCommand::ALLO,
        Arc::new(Box::new(|writer, config, session, arg, _data_stream| {
            Box::pin(crate::core_ftpcommand::allo::handle_allo_command(
                writer, config, session, arg,
            ))
        })),
    );

    handlers.insert(
        FtpCommand::CDUP,
        Arc::new(Box::new(|writer, config, session, arg, _data_stream| {
            Box::pin(crate::core_ftpcommand::cdup::handle_cdup_command(
                writer, config, session, arg,
            ))
        })),
    );

    handlers.insert(
        FtpCommand::FEAT,
        Arc::new(Box::new(|writer, _config, _session, arg, _data_stream| {
            Box::pin(crate::core_ftpcommand::feat::handle_feat_command(
                writer, arg,
            ))
        })),
    );

    handlers.insert(
        FtpCommand::SYST,
        Arc::new(Box::new(|writer, _config, _session, _arg, _data_stream| {
            Box::pin(crate::core_ftpcommand::syst::handle_syst_command(writer))
        })),
    );

    handlers.insert(
        FtpCommand::SITE,
        Arc::new(Box::new(|writer, config, session, arg, _data_stream| {
            Box::pin(crate::core_ftpcommand::site::handle_site_command(
                writer, config, session, arg,
            ))
        })),
    );

    handlers.insert(
        FtpCommand::USER,
        Arc::new(Box::new(|writer, config, session, arg, _data_stream| {
            Box::pin(crate::core_ftpcommand::user::handle_user_command(
                writer, config, session, arg,
            ))
        })),
    );

    handlers.insert(
        FtpCommand::PASS,
        Arc::new(Box::new(|writer, config, session, arg, _data_stream| {
            Box::pin(crate::core_ftpcommand::pass::handle_pass_command(
                writer, config, session, arg,
            ))
        })),
    );

    handlers.insert(
        FtpCommand::QUIT,
        Arc::new(Box::new(|writer, config, _session, arg, _data_stream| {
            Box::pin(crate::core_ftpcommand::quit::handle_quit_command(
                writer,
                config,
                arg.to_string(),
            ))
        })),
    );

    handlers.insert(
        FtpCommand::PWD,
        Arc::new(Box::new(|writer, config, session, arg, _data_stream| {
            Box::pin(crate::core_ftpcommand::pwd::handle_pwd_command(
                writer, config, session, arg,
            ))
        })),
    );

    handlers.insert(
        FtpCommand::LIST,
        Arc::new(Box::new(|writer, config, session, arg, _data_stream| {
            Box::pin(crate::core_ftpcommand::list::handle_list_command(
                writer, config, session, arg,
            ))
        })),
    );

    handlers.insert(
        FtpCommand::CWD,
        Arc::new(Box::new(|writer, config, session, arg, _data_stream| {
            Box::pin(crate::core_ftpcommand::cwd::handle_cwd_command(
                writer, config, session, arg,
            ))
        })),
    );

    handlers.insert(
        FtpCommand::NOOP,
        Arc::new(Box::new(|writer, config, _session, arg, _data_stream| {
            Box::pin(crate::core_ftpcommand::noop::handle_noop_command(
                writer,
                config,
                arg.to_string(),
            ))
        })),
    );

    handlers.insert(
        FtpCommand::MKD,
        Arc::new(Box::new(|writer, config, session, arg, _data_stream| {
            Box::pin(crate::core_ftpcommand::mkd::handle_mkd_command(
                writer, config, session, arg,
            ))
        })),
    );

    handlers.insert(
        FtpCommand::RMD,
        Arc::new(Box::new(|writer, config, session, arg, _data_stream| {
            Box::pin(crate::core_ftpcommand::rmd::handle_rmd_command(
                writer, config, session, arg,
            ))
        })),
    );

    handlers.insert(
        FtpCommand::DELE,
        Arc::new(Box::new(|writer, config, session, arg, _data_stream| {
            Box::pin(crate::core_ftpcommand::dele::handle_dele_command(
                writer, config, session, arg,
            ))
        })),
    );

    handlers.insert(
        FtpCommand::RNFR,
        Arc::new(Box::new(|writer, config, session, arg, _data_stream| {
            Box::pin(crate::core_ftpcommand::rnfr::handle_rnfr_command(
                writer, config, session, arg,
            ))
        })),
    );

    handlers.insert(
        FtpCommand::RNTO,
        Arc::new(Box::new(|writer, config, session, arg, _data_stream| {
            Box::pin(crate::core_ftpcommand::rnto::handle_rnto_command(
                writer, config, session, arg,
            ))
        })),
    );

    handlers.insert(
        FtpCommand::RETR,
        Arc::new(Box::new(|writer, config, session, arg, data_stream| {
            Box::pin(crate::core_ftpcommand::retr::handle_retr_command(
                writer, config, session, arg,
            ))
        })),
    );

    handlers.insert(
        FtpCommand::STOR,
        Arc::new(Box::new(|writer, config, session, arg, data_stream| {
            Box::pin(crate::core_ftpcommand::stor::handle_stor_command(
                writer,
                config,
                session,
                arg,
                data_stream,
            ))
        })),
    );

    handlers.insert(
        FtpCommand::TYPE,
        Arc::new(Box::new(|writer, config, session, arg, _data_stream| {
            Box::pin(crate::core_ftpcommand::type_::handle_type_command(
                writer, config, session, arg,
            ))
        })),
    );

    handlers.insert(
        FtpCommand::PASV,
        Arc::new(Box::new(|writer, config, session, arg, _data_stream| {
            Box::pin(pasv::handle_pasv_command(writer, config, session, arg))
        })),
    );

    handlers.insert(
        FtpCommand::PORT,
        Arc::new(Box::new(|writer, config, session, arg, _data_stream| {
            Box::pin(port::handle_port_command(writer, config, session, arg))
        })),
    );

    // Other commands here !

    handlers
}
