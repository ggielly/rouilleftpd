use eframe::{egui, NativeOptions};

use crate::ipc::Ipc;
use crate::user_record::UserRecord;
use std::sync::Arc;

pub struct App {
    pub ipc: Arc<Ipc>,
    pub records: Vec<UserRecord>,
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _: &mut eframe::Frame) {
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
