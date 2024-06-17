use shared_memory::*;
use std::num::ParseIntError;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum IpcError {
    #[error("Invalid IPC key format")]
    InvalidKeyFormat,
    #[error("Failed to parse IPC key: {0}")]
    ParseError(#[from] ParseIntError),
}

pub struct Ipc {
    pub ipc_key: String,
    #[allow(dead_code)]
    pub memory: Shmem,
}

impl Ipc {
    pub fn new(ipc_key: String) -> Self {
        let memory = match ShmemConf::new().size(1024).os_id(&ipc_key).create() {
            Ok(memory) => memory,
            Err(ShmemError::MappingIdExists) => {
                // If the default ID exists, try to create a new unique ID
                let mut attempt = 1;
                loop {
                    match Self::generate_unique_key(&ipc_key, attempt) {
                        Ok(unique_key) => match ShmemConf::new()
                            .size(1024)
                            .os_id(&unique_key)
                            .create()
                        {
                            Ok(memory) => break memory,
                            Err(ShmemError::MappingIdExists) => {
                                attempt += 1;
                                if attempt > 10 {
                                    panic!("Failed to create unique shared memory segment after multiple attempts.");
                                }
                            }
                            Err(e) => panic!("Failed to create shared memory: {:?}", e),
                        },
                        Err(e) => panic!("Failed to generate unique key: {:?}", e),
                    }
                }
            }
            Err(e) => panic!("Failed to create shared memory: {:?}", e),
        };

        Self { ipc_key, memory }
    }

    fn generate_unique_key(ipc_key: &str, attempt: u32) -> Result<String, IpcError> {
        if !ipc_key.starts_with("0x") {
            return Err(IpcError::InvalidKeyFormat);
        }

        let key_num = u32::from_str_radix(&ipc_key[2..], 16)?;
        let unique_key = format!("0x{:08X}", key_num + attempt);
        Ok(unique_key)
    }
}
