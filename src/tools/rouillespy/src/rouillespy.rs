use std::num::ParseIntError;
use std::sync::{Arc, Mutex};
use std::{env, process};
use thiserror::Error;

pub mod ipcspy;
use crate::ipcspy::UserRecord;

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
    pub fn new(ipc_key: String) -> Result<Self, IpcError> {
        // Validate the IPC key
        if !ipc_key.starts_with("0x") || ipc_key.len() != 10 {
            return Err(IpcError::InvalidKeyFormat);
        }

        // Simulate shared memory with a Vec<u8>
        let memory = Arc::new(Mutex::new(vec![0; 1024]));

        Ok(Self { ipc_key, memory })
    }

    fn generate_unique_key(ipc_key: &str, attempt: u32) -> Result<String, IpcError> {
        if !ipc_key.starts_with("0x") {
            return Err(IpcError::InvalidKeyFormat);
        }

        let key_num = u32::from_str_radix(&ipc_key[2..], 16)?;
        let unique_key = format!("{:08X}", key_num + attempt);
        Ok(unique_key)
    }

    pub fn read_user_records(&self) -> Vec<UserRecord> {
        let memory = self.memory.lock().unwrap();
        let mut records = Vec::new();

        let record_size = std::mem::size_of::<UserRecord>();
        let num_records = memory.len() / record_size;

        for i in 0..num_records {
            let offset = i * record_size;
            let record: UserRecord = unsafe {
                std::ptr::read(memory[offset..].as_ptr() as *const UserRecord)
            };
            records.push(record);
        }

        records
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        eprintln!("Usage: {} <IPC_KEY>", args[0]);
        process::exit(1);
    }

    let ipc_key = &args[1];
    let ipc = match Ipc::new(ipc_key.to_string()) {
        Ok(ipc) => ipc,
        Err(e) => {
            eprintln!("Failed to initialize IPC: {}", e);
            process::exit(1);
        }
    };

    let records = ipc.read_user_records();
    for record in records {
        let username = String::from_utf8_lossy(&record.username).trim_end_matches('\0').to_string();
        let command = String::from_utf8_lossy(&record.command).trim_end_matches('\0').to_string();
        println!(
            "Username: {}, Command: {}, Download Speed: {:.2} KB/s, Upload Speed: {:.2} KB/s",
            username, command, record.download_speed, record.upload_speed
        );
    }
}