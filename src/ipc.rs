use shared_memory::*;
use std::num::ParseIntError;
use thiserror::Error;
use std::sync::{Arc, Mutex};

#[derive(Debug, Error)]
pub enum IpcError {
    #[error("Invalid IPC key format")]
    InvalidKeyFormat,
    #[error("Failed to parse IPC key: {0}")]
    ParseError(#[from] ParseIntError),
}

#[derive(Clone)]
pub struct Ipc {
    pub ipc_key: String,
    pub memory: Arc<Mutex<Vec<u8>>>, // Use a Vec<u8> wrapped in Arc<Mutex>> for shared memory
}

impl Ipc {
    pub fn new(ipc_key: String) -> Self {
        // Simulate shared memory with a Vec<u8>
        let memory = Arc::new(Mutex::new(vec![0; 1024]));

        Self { ipc_key, memory }
    }

    fn generate_unique_key(ipc_key: &str, attempt: u32) -> Result<String, IpcError> {
        if !ipc_key.starts_with("0x") {
            return Err(IpcError::InvalidKeyFormat);
        }

        let key_num = u32::from_str_radix(&ipc_key[2..], 16)?;
        let unique_key = format!("{:08X}", key_num + attempt);
        Ok(unique_key)
    }
}
