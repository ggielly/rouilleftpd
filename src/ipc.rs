use crate::users::UserRecord;
use log::{debug, warn};
use log::{error, trace};
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
    pub memory: Arc<Mutex<Vec<u8>>>, // Use a Vec<u8> wrapped in Arc<Mutex>> for shared memory
}

impl Ipc {
    pub fn new(ipc_key: String) -> Result<Self, String> {
        if ipc_key.is_empty() {
            Err("IPC key is empty.".to_string())
        } else {
            Ok(Ipc {
                ipc_key,
                memory: Arc::new(Mutex::new(Vec::new())),
            })
        }
    }

    pub fn read_user_records(&self) -> Vec<UserRecord> {
        let memory = self.memory.lock().unwrap();
        let mut records = Vec::new();

        for chunk in memory.chunks_exact(72) {
            // Each record is 72 bytes
            let record = UserRecord::from_bytes(chunk);
            records.push(record);
        }

        // Debug output while reading
        trace!("Read user records: {:?}", records);

        records
    }

    pub fn write_user_record(&self, record: UserRecord) {
        let mut memory = self.memory.lock().unwrap();
        let bytes = record.to_bytes();

        // Ensure the memory vector is large enough
        if memory.len() < bytes.len() {
            warn!(
                "Memory is too small: the len is {} but required len is {}. Resizing the memory.",
                memory.len(),
                bytes.len()
            );
            memory.resize(bytes.len(), 0);
        }

        // Writing record to the shared memory
        for (i, &byte) in bytes.iter().enumerate() {
            memory[i] = byte;
        }

        // Debug output after writing
        trace!("Memory after write operation: {:?}", *memory);

        // Eject the written bytes from memory and verify correctness
        let read_back = &memory[0..bytes.len()];
        trace!("Bytes read back from memory: {:?}", read_back);

        // Additional log to check the record during writing
        trace!("Record written: {:?}", record);
    }
}

pub fn update_ipc(
    ipc: Arc<Ipc>,
    username: &str,
    command: &str,
    download_speed: f32,
    upload_speed: f32,
) {
    let mut user_record = UserRecord {
        username: [0; 32],
        command: [0; 32],
        download_speed,
        upload_speed,
    };

    // Copy the username and command into the fixed-size arrays
    let username_bytes = username.as_bytes();
    let command_bytes = command.as_bytes();
    user_record.username[..username_bytes.len()].copy_from_slice(username_bytes);
    user_record.command[..command_bytes.len()].copy_from_slice(command_bytes);

    ipc.write_user_record(user_record);
}
