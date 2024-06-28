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

/*impl Ipc {
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
}*/

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct UserRecord {
    pub username: [u8; 32],  // Fixed-size array for username (32 bytes)
    pub command: [u8; 32],   // Fixed-size array for command (32 bytes)
    pub download_speed: f32, // Download speed
    pub upload_speed: f32,   // Upload speed
}

impl UserRecord {
    fn from_bytes(bytes: &[u8]) -> Self {
        let username = {
            let mut array = [0u8; 32];
            array.copy_from_slice(&bytes[0..32]);
            array
        };
        let command = {
            let mut array = [0u8; 32];
            array.copy_from_slice(&bytes[32..64]);
            array
        };
        let download_speed = f32::from_ne_bytes([bytes[64], bytes[65], bytes[66], bytes[67]]);
        let upload_speed = f32::from_ne_bytes([bytes[68], bytes[69], bytes[70], bytes[71]]);

        UserRecord {
            username,
            command,
            download_speed,
            upload_speed,
        }
    }

    fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&self.username);
        bytes.extend_from_slice(&self.command);
        bytes.extend_from_slice(&self.download_speed.to_ne_bytes());
        bytes.extend_from_slice(&self.upload_speed.to_ne_bytes());
        bytes
    }
}

impl Ipc {
    pub fn new(ipc_key: String) -> Self {
        // Simulate shared memory with a Vec<u8>
        let memory = Arc::new(Mutex::new(vec![0; 1024])); // Adjust size as needed

        Self { ipc_key, memory }
    }

    pub fn write_user_record(&self, record: UserRecord) {
        let mut memory = self.memory.lock().unwrap();
        let bytes = record.to_bytes();
        for (i, &byte) in bytes.iter().enumerate() {
            memory[i] = byte;
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

        records
    }
}
