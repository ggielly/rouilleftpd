use clap::{Arg, Command};
use eframe::{egui, NativeOptions};
use log::{debug, error, info, trace};
use std::{
    num::ParseIntError,
    sync::{Arc, Mutex},
    thread,
    time::Duration,
};
use thiserror::Error;

mod ipcspy;
//use ipcspy::UserRecord;

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
    pub fn new(ipc_key: String) -> Result<Self, IpcError> {
        trace!("Creating new IPC instance with key: {}", ipc_key);

        if !ipc_key.starts_with("0x") {
            error!("Invalid IPC key format: {}", ipc_key);
            return Err(IpcError::InvalidKeyFormat);
        }

        u32::from_str_radix(&ipc_key[2..], 16)?; // Validate key format

        let memory = Arc::new(Mutex::new(vec![0; 1024])); // Simulate shared memory

        info!("IPC instance created with key: {}", ipc_key);
        Ok(Self { ipc_key, memory })
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

struct App {
    ipc: Ipc,
    records: Vec<UserRecord>,
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _: &mut eframe::Frame) {
        trace!("Updating GUI with user records");

        self.records = self.ipc.read_user_records();

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("User Records");

            egui::Grid::new("user_records")
                .num_columns(4)
                .striped(true)
                .show(ui, |ui| {
                    ui.label("Username");
                    ui.label("Command");
                    ui.label("Download Speed");
                    ui.label("Upload Speed");
                    ui.end_row();

                    for record in &self.records {
                        ui.label(String::from_utf8_lossy(&record.username));
                        ui.label(String::from_utf8_lossy(&record.command));
                        ui.label(format!("{:.2} KB/s", record.download_speed));
                        ui.label(format!("{:.2} KB/s", record.upload_speed));
                        ui.end_row();
                    }
                });
        });
    }
}

fn main() -> Result<(), eframe::Error> {
    env_logger::init();

    let matches = Command::new("RouilleSpy")
        .version("0.01")
        .author("thegug")
        .about("Displays user records from IPC memory")
        .arg(
            Arg::new("ipc_key")
                .short('k')
                .long("key")
                .value_name("IPC_KEY")
                .help("Sets the IPC key (e.g., 0x0000DEAD)")
                .default_value("0x0000DEAD")
                .value_parser(clap::value_parser!(String)),
        )
        .arg(
            Arg::new("gui")
                .short('g')
                .long("gui")
                .help("Run in GUI mode")
                .action(clap::ArgAction::SetTrue),
        )
        .get_matches();

    let ipc_key = matches.get_one::<String>("ipc_key").cloned().unwrap();
    let ipc = Ipc::new(ipc_key).expect("Failed to create IPC instance");

    if matches.get_flag("gui") {
        info!("Running in GUI mode");

        let app = App {
            ipc,
            records: Vec::new(),
        };
        let native_options = NativeOptions::default();

        match eframe::run_native(
            "RouilleSpy GUI",
            native_options,
            Box::new(|_cc| Box::new(app)),
        ) {
            Ok(_) => info!("GUI application exited successfully"),
            Err(e) => error!("Failed to run GUI application: {}", e),
        }
    } else {
        info!("Running in CLI mode");
        loop {
            let records = ipc.read_user_records();
            if records.is_empty() {
                info!("No user records found in IPC memory");
            }
            for record in records {
                println!(
                    "Username: {}, Command: {}, Download Speed: {:.2} KB/s, Upload Speed: {:.2} KB/s",
                    String::from_utf8_lossy(&record.username),
                    String::from_utf8_lossy(&record.command),
                    record.download_speed,
                    record.upload_speed
                );
            }
            thread::sleep(Duration::from_secs(1)); // Update every 1 second
        }
    }

    Ok(())
}
