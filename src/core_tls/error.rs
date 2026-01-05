// Gestion des erreurs pour le module TLS
use thiserror::Error;

#[derive(Error, Debug)]
pub enum TlsError {
    #[error("Failed to load SSL certificate: {0}")]
    CertificateLoadError(String),

    #[error("Failed to load SSL private key: {0}")]
    PrivateKeyLoadError(String),

    #[error("Failed to create TLS acceptor: {0}")]
    TlsAcceptorError(String),

    #[error("TLS handshake failed: {0}")]
    TlsHandshakeError(String),

    #[error("TLS configuration error: {0}")]
    TlsConfigError(String),

    #[error("TLS not configured")]
    TlsNotConfigured,
}

impl TlsError {
    pub fn to_ftp_response(&self) -> String {
        match self {
            TlsError::TlsNotConfigured => {
                "534 TLS not available. Please configure SSL/TLS in the server.".to_string()
            }
            _ => "451 Requested action aborted. Local error in processing.".to_string(),
        }
    }
}
