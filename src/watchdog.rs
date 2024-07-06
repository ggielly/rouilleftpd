use crate::ipc::Ipc;
use log::info;
use std::{sync::Arc, thread, time::Duration};

pub fn start_watchdog(ipc: Arc<Ipc>, verbose: bool) {
    thread::spawn(move || {
        loop {
            {
                let records = ipc.read_user_records();
                for record in records {
                    if verbose {
                        info!(
                            "Username: {}, Command: {}, Download Speed: {:.2} KB/s, Upload Speed: {:.2} KB/s",
                            String::from_utf8_lossy(&record.username),
                            String::from_utf8_lossy(&record.command),
                            record.download_speed,
                            record.upload_speed
                        );
                    }
                }
            }
            thread::sleep(Duration::from_secs(50)); // Update every 50 second
        }
    });
}
