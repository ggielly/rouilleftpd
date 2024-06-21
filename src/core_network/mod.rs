pub mod network;
pub mod pasv;
pub mod port;

use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::net::TcpStream;

pub struct Session {
    pub current_dir: String,
    pub rename_from: Option<PathBuf>,
    pub data_stream: Option<Arc<Mutex<TcpStream>>>,
}

impl Session {
    pub fn new() -> Self {
        Self {
            current_dir: String::from("/"),
            rename_from: None,
            data_stream: None,
        }
    }
}
