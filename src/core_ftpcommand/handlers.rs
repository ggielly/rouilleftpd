use crate::core_network::Session;
use crate::Config;
use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use tokio::net::TcpStream;
use tokio::sync::Mutex;

// Specific crated for PORT and PASV commands
use crate::core_network::port;
use crate::core_network::pasv;

type BoxedHandler = Box<
    dyn Fn(
            Arc<Mutex<TcpStream>>,
            Arc<Config>,
            Arc<Mutex<Session>>,
            String,
        ) -> Pin<Box<dyn Future<Output = Result<(), std::io::Error>> + Send>>
        + Send
        + Sync,
>;

pub fn initialize_command_handlers() -> HashMap<String, Arc<BoxedHandler>> {
    let mut handlers: HashMap<String, Arc<BoxedHandler>> = HashMap::new();

    handlers.insert(
        "USER".to_string(),
        Arc::new(Box::new(|writer, config, _session, arg| {
            Box::pin(crate::core_ftpcommand::user::handle_user_command(
                writer,
                config,
                arg.to_string(),
            ))
        })),
    );
    handlers.insert(
        "PASS".to_string(),
        Arc::new(Box::new(|writer, config, _session, arg| {
            Box::pin(crate::core_ftpcommand::pass::handle_pass_command(
                writer,
                config,
                arg.to_string(),
            ))
        })),
    );
    handlers.insert(
        "QUIT".to_string(),
        Arc::new(Box::new(|writer, config, _session, arg| {
            Box::pin(crate::core_ftpcommand::quit::handle_quit_command(
                writer,
                config,
                arg.to_string(),
            ))
        })),
    );
    handlers.insert(
        "PWD".to_string(),
        Arc::new(Box::new(|writer, config, session, arg| {
            Box::pin(crate::core_ftpcommand::pwd::handle_pwd_command(
                writer,
                config,
                session,
                arg.to_string(),
            ))
        })),
    );
    handlers.insert(
        "LIST".to_string(),
        Arc::new(Box::new(|writer, config, session, arg| {
            Box::pin(crate::core_ftpcommand::list::handle_list_command(
                writer,
                config,
                session,
                arg.to_string(),
            ))
        })),
    );
    handlers.insert(
        "CWD".to_string(),
        Arc::new(Box::new(|writer, config, session, arg| {
            Box::pin(crate::core_ftpcommand::cwd::handle_cwd_command(
                writer,
                config,
                session,
                arg.to_string(),
            ))
        })),
    );

    handlers.insert(
        "NOOP".to_string(),
        Arc::new(Box::new(|writer, config, _session, arg| {
            Box::pin(crate::core_ftpcommand::noop::handle_noop_command(
                writer,
                config,
                arg.to_string(),
            ))
        })),
    );

    handlers.insert(
        "MKD".to_string(),
        Arc::new(Box::new(|writer, config, session, arg| {
            Box::pin(crate::core_ftpcommand::mkd::handle_mkd_command(
                writer,
                config,
                session,
                arg.to_string(),
            ))
        })),
    );

    handlers.insert(
        "RMD".to_string(),
        Arc::new(Box::new(|writer, config, session, arg| {
            Box::pin(crate::core_ftpcommand::rmd::handle_rmd_command(
                writer,
                config,
                session,
                arg.to_string(),
            ))
        })),
    );

    handlers.insert(
        "DELE".to_string(),
        Arc::new(Box::new(|writer, config, session, arg| {
            Box::pin(crate::core_ftpcommand::dele::handle_dele_command(
                writer,
                config,
                session,
                arg.to_string(),
            ))
        })),
    );

    handlers.insert(
        "RNFR".to_string(),
        Arc::new(Box::new(|writer, config, session, arg| {
            Box::pin(crate::core_ftpcommand::rnfr::handle_rnfr_command(
                writer,
                config,
                session,
                arg.to_string(),
            ))
        })),
    );
    handlers.insert(
        "RNTO".to_string(),
        Arc::new(Box::new(|writer, config, session, arg| {
            Box::pin(crate::core_ftpcommand::rnto::handle_rnto_command(
                writer,
                config,
                session,
                arg.to_string(),
            ))
        })),
    );

    handlers.insert(
        "RETR".to_string(),
        Arc::new(Box::new(|writer, config, session, arg| {
            Box::pin(crate::core_ftpcommand::retr::handle_retr_command(
                writer,
                config,
                session,
                arg.to_string(),
            ))
        })),
    );

    handlers.insert(
        "STOR".to_string(),
        Arc::new(Box::new(|writer, config, session, arg| {
            Box::pin(async move {
                // Create a data connection here
                let data_stream = Arc::new(Mutex::new(TcpStream::connect("localhost:20").await.unwrap()));
                crate::core_ftpcommand::stor::handle_stor_command(
                    writer,
                    data_stream,
                    config,
                    session,
                    arg.to_string(),
                ).await
            })
        })),
    );

    handlers.insert(
        "PORT".to_string(),
        Arc::new(Box::new(|writer, config, session, arg| {
            Box::pin(port::handle_port_command(
                writer,
                config,
                session,
                arg.to_string(),
            ))
        })),
    );

    handlers.insert(
        "PASV".to_string(),
        Arc::new(Box::new(|writer, config, session, arg| {
            Box::pin(pasv::handle_pasv_command(
                writer,
                config,
                session,
                arg.to_string(),
            ))
        })),
    );


    handlers
}
