use std::num::ParseIntError;
use std::sync::{Arc, Mutex};
use std::{env, process};
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
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        eprintln!("Usage: {} <IPC_KEY>", args[0]);
        process::exit(1);
    }

    let ipc_key = &args[1];
    match Ipc::new(ipc_key.to_string()) {
        Ok(ipc) => {
            let memory = ipc.memory.lock().unwrap();
            for (i, byte) in memory.iter().enumerate() {
                print!("{:02X} ", byte);
                if (i + 1) % 16 == 0 {
                    println!();
                }
            }
        }
        Err(e) => {
            eprintln!("Error initializing IPC: {}", e);
            process::exit(1);
        }
    }
}
