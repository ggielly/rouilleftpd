use log::debug;
use log::trace;

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
        trace!("Converting bytes to UserRecord");

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

        debug!(
            "UserRecord created: username={}, command={}, download_speed={}, upload_speed={}",
            String::from_utf8_lossy(&username),
            String::from_utf8_lossy(&command),
            download_speed,
            upload_speed
        );

        UserRecord {
            username,
            command,
            download_speed,
            upload_speed,
        }
    }
}
