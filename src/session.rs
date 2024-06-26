use std::path::PathBuf;
use std::sync::Arc;
use sysinfo::{DiskExt, System, SystemExt};
use tokio::net::TcpStream;
use tokio::sync::Mutex;

/// DEBUG - REMOVE ME ///
///
use rand::Rng;
///

pub struct Session {
    pub current_dir: String,
    pub rename_from: Option<PathBuf>,
    pub data_stream: Option<Arc<Mutex<TcpStream>>>,
    pub type_: String,            // The primary transfer type (A, E, I, L)
    pub byte_size: Option<u8>,    // The byte size for TYPE L (None if not applicable)
    pub base_path: PathBuf,       // chroot_dir + min_dir
    pub username: Option<String>, // Username for the session
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
            username: None,         // Initialize username as None
        }
    }

    // Returns the average transfer rate.
    ///
    /// This is a placeholder implementation that generates a random value to simulate
    /// the average transfer rate.
    /// TODO : Replace this with the right operation
    pub fn average_transfer_rate(&self) -> f64 {
        let mut rng = rand::thread_rng();
        rng.gen_range(0.0..1000.0) // Generates a random number between 0 and 1000
    }

    /// Returns the free space in the current directory in MiB.
    ///
    /// This method calculates the free space available on the disk where the current
    /// directory is located and returns it in Mebibytes (MiB).
    pub fn free_space_mib(&self) -> f64 {
        let sys = System::new_all();
        let path = &self.base_path;

        for disk in sys.disks() {
            if path.starts_with(disk.mount_point()) {
                return disk.available_space() as f64 / 1_048_576.0; // Convert bytes to MiB
            }
        }

        0.0  // sic(!)
    }

    pub fn upload_stat(&self) -> (f64, &str) {
        (0.0, "MB") // Example values, replace with actual logic
    }

    pub fn download_stat(&self) -> (f64, &str) {
        (0.0, "MB") // Example values, replace with actual logic
    }

    pub fn speed_stat(&self) -> (f64, &str) {
        (7181.07, "K/s") // Example values, replace with actual logic
    }

    pub fn section(&self) -> &str {
        "DEFAULT" // Example value, replace with actual logic
    }

    pub fn credits(&self) -> (f64, &str) {
        (14.6, "MB") // Example values, replace with actual logic
    }

    pub fn ratio(&self) -> &str {
        "Unlimited" // Example value, replace with actual logic
    }
}
