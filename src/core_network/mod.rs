use std::path::PathBuf;

pub mod network;

pub struct Session {
    pub current_dir: String,
    pub rename_from: Option<PathBuf>,
}

impl Session {
    pub fn new() -> Self {
        Self {
            current_dir: String::from("/"),
            rename_from: None,
        }
    }
}