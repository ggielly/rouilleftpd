use crate::Ipc;

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct UserRecord {
    pub username: [u8; 32],  // Fixed-size array for username (32 bytes)
    pub command: [u8; 32],   // Fixed-size array for command (32 bytes)
    pub download_speed: f32, // Download speed
    pub upload_speed: f32,   // Upload speed
}

impl UserRecord {
    pub fn from_bytes(bytes: &[u8]) -> Self {
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

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&self.username);
        bytes.extend_from_slice(&self.command);
        bytes.extend_from_slice(&self.download_speed.to_ne_bytes());
        bytes.extend_from_slice(&self.upload_speed.to_ne_bytes());
        bytes
    }
}


// Alpha version
pub fn update_user_record(
    ipc: &Ipc,
    username: &str,
    command: &str,
    download_speed: f32,
    upload_speed: f32,
) {
    let mut username_bytes = [0u8; 32];
    let mut command_bytes = [0u8; 32];

    // Copy the username and command into the fixed-size arrays
    username_bytes[..username.len()].copy_from_slice(username.as_bytes());
    command_bytes[..command.len()].copy_from_slice(command.as_bytes());

    // Create a new UserRecord instance with the provided data
    let record = UserRecord {
        username: username_bytes,
        command: command_bytes,
        download_speed,
        upload_speed,
    };

    // Debug output before writing
    eprintln!("Creating user record: {:?}", record);

    // Write the UserRecord instance to the shared memory
    ipc.write_user_record(record);

    // Ensure the write operation
    eprintln!("User record updated: {:?}", record);
    eprintln!("Shared memory: {:?}", ipc.memory);
}