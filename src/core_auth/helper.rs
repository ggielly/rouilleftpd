use crate::core_auth::core_auth::PasswdEntry;
use bcrypt::{hash, verify, DEFAULT_COST};
use log::{error, info};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::io::AsyncWriteExt;
use tokio::net::TcpStream;
use tokio::sync::Mutex;

pub fn hash_password(password: &str) -> String {
    hash(password, DEFAULT_COST).expect("Failed to hash password")
}

pub fn verify_password(password: &str, hashed_password: &str) -> bool {
    verify(password, hashed_password).unwrap_or(false)
}

pub async fn load_passwd_file(path: &str) -> HashMap<String, PasswdEntry> {
    let mut passwd_map = HashMap::new();
    let content = fs::read_to_string(path).expect("Failed to read passwd file");

    for line in content.lines() {
        if let Some(entry) = PasswdEntry::from_line(line) {
            passwd_map.insert(entry.get_username().to_string(), entry);
        }
    }
    passwd_map
}
