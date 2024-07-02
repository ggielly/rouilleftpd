use crate::user_record::UserRecord;
use log::{debug, error, info, trace};
use std::num::ParseIntError;
use std::sync::{Arc, Mutex};
use thiserror::Error;

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
    pub memory: Arc<Mutex<Vec<u8>>>,
}

impl Ipc {
    pub fn new(ipc_key: &str) -> Result<Self, IpcError> {
        trace!("Creating new IPC instance with key: {}", ipc_key);

        if !ipc_key.starts_with("0x") {
            error!("Invalid IPC key format: {}", ipc_key);
            return Err(IpcError::InvalidKeyFormat);
        }

        u32::from_str_radix(&ipc_key[2..], 16)?; // Validate key format

        let memory = Arc::new(Mutex::new(vec![0; 1024])); // Simulate shared memory

        info!("IPC instance created with key: {}", ipc_key);
        Ok(Self {
            ipc_key: ipc_key.to_string(),
            memory,
        })
    }

    pub fn read_user_records(&self) -> Vec<UserRecord> {
        trace!("Reading user records from IPC memory");

        let memory = self.memory.lock().unwrap();
        let mut records = Vec::new();

        for chunk in memory.chunks_exact(72) {
            // Each record is 72 bytes
            if chunk.iter().any(|&byte| byte != 0) {
                // Skip empty records
                let record = UserRecord::from_bytes(chunk);
                records.push(record);
            }
        }

        debug!("Read {} user records", records.len());
        records
    }
}
