pub mod network;

pub struct Session {
    pub current_dir: String,
}

impl Session {
    pub fn new() -> Self {
        Self {
            current_dir: String::from("/"),
        }
    }
}