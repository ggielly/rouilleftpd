#[derive(Debug, Clone)]
pub struct PasswdEntry {
    username: String,
    hashed_password: String,
}

impl PasswdEntry {
    pub fn from_line(line: &str) -> Option<Self> {
        let parts: Vec<&str> = line.split(':').collect();
        if parts.len() != 2 {
            return None;
        }
        let entry = PasswdEntry {
            username: parts[0].to_string(),
            hashed_password: parts[1].to_string(),
        };

        Some(entry)
    }

    pub fn get_hashed_password(&self) -> &str {
        &self.hashed_password
    }

    pub fn get_username(&self) -> &str {
        &self.username
    }
}
