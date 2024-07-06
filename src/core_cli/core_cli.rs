use clap::Parser;

/// Command-line arguments
#[derive(Parser, Debug)]
#[command(name = "rouilleftpd", about = "A FTP server written in Rust.")]
pub struct Cli {
    /// Path to the configuration file
    #[arg(short, long, default_value = "")]
    pub config: String,

    /// IPC key for shared memory
    #[arg(short, long)]
    pub ipc_key: Option<String>,

    /// Enable verbose mode
    #[arg(short, long)]
    pub verbose: bool,
}
