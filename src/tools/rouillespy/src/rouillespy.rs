mod app;
mod cli;
mod ipc;
mod user_record;

use cli::run_cli;
use eframe::NativeOptions;
use log::info;
use std::sync::Arc;

fn main() -> Result<(), eframe::Error> {
    let matches = cli::get_matches();

    // Set log level based on verbosity argument
    let verbosity = matches.get_one::<String>("verbosity").unwrap();
    std::env::set_var("RUST_LOG", verbosity);
    env_logger::init();

    let ipc_key = matches.get_one::<String>("ipc_key").cloned().unwrap();
    let ipc = ipc::Ipc::new(&ipc_key).expect("Failed to create IPC instance");

    info!("Starting RouilleSpy with IPC key: {}", ipc_key);

    if matches.contains_id("gui") {
        // GUI Mode
        info!("Running in GUI mode");
        let app = app::App {
            ipc: Arc::new(ipc),
            records: Vec::new(),
        };
        let native_options = NativeOptions::default();
        eframe::run_native(
            "RouilleSpy GUI",
            native_options,
            Box::new(|_cc| Box::new(app)),
        )?;
    } else {
        // CLI Mode
        info!("Running in CLI mode");
        run_cli(Arc::new(ipc));
    }

    Ok(())
}
