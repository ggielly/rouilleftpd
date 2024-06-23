pub mod network;
pub mod pasv;
pub mod port;

use std::path::PathBuf;
use std::sync::Arc;
use tokio::net::TcpStream;
use tokio::sync::Mutex;

pub struct Session {
    pub current_dir: String,
    pub rename_from: Option<PathBuf>,
    pub data_stream: Option<Arc<Mutex<TcpStream>>>,
    pub type_: String,         // The primary transfer type (A, E, I, L)
    pub byte_size: Option<u8>, // The byte size for TYPE L (None if not applicable)
    pub base_path: PathBuf,    // chroot_dir + min_dir
}

impl Session {
    pub fn new(base_path: PathBuf) -> Self {
        Self {
            current_dir: String::from("/"),
            base_path,
            rename_from: None,
            data_stream: None,
            type_: "A".to_string(), // Default transfer type is ASCII
            byte_size: None,        // Default byte size is None
        }
    }
}
