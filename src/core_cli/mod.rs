use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(name = "rouilleftpd", about = "A FTP server written in Rust.")]
pub struct Cli {
    /// Path to the configuration file
    #[structopt(short, long, default_value = "")]
    pub config: String,

    /// IPC key for shared memory
    #[structopt(short, long)]
    pub ipc_key: Option<String>,

    // Additional command-line options can be added here
}
