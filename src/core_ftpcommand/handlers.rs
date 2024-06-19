use tokio::io::AsyncWriteExt;
use tokio::net::TcpStream;
use tokio::sync::Mutex;
use std::sync::Arc;
use crate::Config;
use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;

type BoxedHandler = Arc<dyn Fn(Arc<Mutex<TcpStream>>, Arc<Config>, String) -> Pin<Box<dyn Future<Output = Result<(), std::io::Error>> + Send>> + Send + Sync>;

pub fn initialize_command_handlers() -> HashMap<String, BoxedHandler> {
    let mut handlers: HashMap<String, BoxedHandler> = HashMap::new();

    handlers.insert("USER".to_string(), Arc::new(|writer, config, arg| {
        Box::pin(crate::core_ftpcommand::user::handle_user_command(writer, config, arg.to_string()))
    }));
    handlers.insert("PASS".to_string(), Arc::new(|writer, config, arg| {
        Box::pin(crate::core_ftpcommand::pass::handle_pass_command(writer, config, arg.to_string()))
    }));
    handlers.insert("QUIT".to_string(), Arc::new(|writer, config, arg| {
        Box::pin(crate::core_ftpcommand::quit::handle_quit_command(writer, config, arg.to_string()))
    }));
    handlers.insert("PWD".to_string(), Arc::new(|writer, config, arg| {
        Box::pin(crate::core_ftpcommand::pwd::handle_pwd_command(writer, config, arg.to_string()))
    }));
    // Add more handlers as needed

    handlers
}
