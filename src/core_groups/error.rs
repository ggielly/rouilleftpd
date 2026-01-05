// Gestion des erreurs pour le module de groupes
use thiserror::Error;

#[derive(Error, Debug)]
pub enum GroupError {
    #[error("Group not found: {0}")]
    GroupNotFound(String),

    #[error("User not found: {0}")]
    UserNotFound(String),

    #[error("Failed to read group data: {0}")]
    GroupReadError(String),

    #[error("Failed to write group data: {0}")]
    GroupWriteError(String),

    #[error("Permission denied: {0}")]
    PermissionDenied(String),

    #[error("Invalid group configuration: {0}")]
    InvalidGroupConfig(String),
}

impl GroupError {
    pub fn to_ftp_response(&self) -> String {
        match self {
            GroupError::GroupNotFound(_) => "550 Group not found.".to_string(),
            GroupError::UserNotFound(_) => "550 User not found.".to_string(),
            GroupError::PermissionDenied(_) => "550 Permission denied.".to_string(),
            _ => "451 Requested action aborted. Local error in processing.".to_string(),
        }
    }
}
