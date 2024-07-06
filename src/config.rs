use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct ServerConfig {
    pub listen_port: u16,
    pub pasv_address: String,
    pub ipc_key: String,
    pub chroot_dir: String,
    pub min_homedir: String,
    pub upload_buffer_size: Option<usize>, // Optional to allow default value
    pub download_buffer_size: Option<usize>, // Optional to allow default value
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Config {
    pub server: ServerConfig,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            listen_port: 21,
            pasv_address: String::from("0.0.0.0"),
            ipc_key: String::from("default_key"),
            chroot_dir: String::from("/var/ftp"),
            min_homedir: String::from("/home/ftp"),
            upload_buffer_size: Some(256 * 1024), // Default 256 KB
            download_buffer_size: Some(128 * 1024), // Default 128 KB
        }
    }
}

impl Config {
    pub fn load_from_file(path: &str) -> Self {
        let config_str = std::fs::read_to_string(path).expect("Failed to read config file");
        let mut config: Config = toml::from_str(&config_str).expect("Failed to parse config file");

        // Set defaults if not specified
        if config.server.upload_buffer_size.is_none() {
            config.server.upload_buffer_size = Some(256 * 1024);
        }
        if config.server.download_buffer_size.is_none() {
            config.server.download_buffer_size = Some(128 * 1024);
        }

        config
    }
}
