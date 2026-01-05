// Gestion des erreurs pour le module de quota/ratio
use log::error;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum QuotaError {
    #[error("Quota exceeded for user {0}")]
    QuotaExceeded(String),

    #[error("Ratio limit reached for user {0}")]
    RatioLimitReached(String),

    #[error("Invalid quota configuration: {0}")]
    InvalidQuotaConfig(String),

    #[error("Invalid ratio configuration: {0}")]
    InvalidRatioConfig(String),

    #[error("Failed to read quota data: {0}")]
    QuotaReadError(String),

    #[error("Failed to write quota data: {0}")]
    QuotaWriteError(String),

    #[error("User not found: {0}")]
    UserNotFound(String),

    #[error("Group not found: {0}")]
    GroupNotFound(String),
}

impl QuotaError {
    pub fn to_ftp_response(&self) -> String {
        match self {
            QuotaError::QuotaExceeded(_) => {
                "552 Requested file action aborted. Exceeded storage allocation.".to_string()
            }
            QuotaError::RatioLimitReached(_) => {
                "552 Requested file action aborted. Ratio limit reached.".to_string()
            }
            _ => "451 Requested action aborted. Local error in processing.".to_string(),
        }
    }
}
