// src/core_ftpcommand/handlers.rs
use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use tokio::net::TcpStream;
use tokio::sync::Mutex;
use crate::Config;

type BoxedHandler = Box<dyn Fn(Arc<Mutex<TcpStream>>, Arc<Config>, String) -> Pin<Box<dyn Future<Output = Result<(), std::io::Error>> + Send>> + Send + Sync>;

pub fn initialize_command_handlers() -> HashMap<String, Arc<BoxedHandler>> {
    let mut handlers: HashMap<String, Arc<BoxedHandler>> = HashMap::new();

    handlers.insert("USER".to_string(), Arc::new(Box::new(|writer, config, arg| {
        Box::pin(crate::core_ftpcommand::user::handle_user_command(writer, config, arg.to_string()))
    })));
    handlers.insert("PASS".to_string(), Arc::new(Box::new(|writer, config, arg| {
        Box::pin(crate::core_ftpcommand::pass::handle_pass_command(writer, config, arg.to_string()))
    })));
    handlers.insert("QUIT".to_string(), Arc::new(Box::new(|writer, config, arg| {
        Box::pin(crate::core_ftpcommand::quit::handle_quit_command(writer, config, arg.to_string()))
    })));
    handlers.insert("PWD".to_string(), Arc::new(Box::new(|writer, config, arg| {
        Box::pin(crate::core_ftpcommand::pwd::handle_pwd_command(writer, config, arg.to_string()))
    })));

    handlers.insert("LIST".to_string(), Arc::new(Box::new(|writer, config, arg| {
        Box::pin(crate::core_ftpcommand::list::handle_list_command(writer, config, arg.to_string()))
    })));

    handlers
}
