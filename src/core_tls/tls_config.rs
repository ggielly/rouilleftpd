// Configuration TLS pour rouilleftpd
use crate::core_tls::error::TlsError;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TlsConfig {
    /// Activation du support TLS
    pub enabled: bool,

    /// Chemin vers le certificat SSL
    pub cert_file: PathBuf,

    /// Chemin vers la clé privée SSL
    pub key_file: PathBuf,

    /// Activation du TLS implicite (FTPS)
    pub implicit_tls: bool,

    /// Port pour les connexions TLS implicites
    pub implicit_tls_port: u16,
}

impl Default for TlsConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            cert_file: PathBuf::from("etc/ssl/cert.pem"),
            key_file: PathBuf::from("etc/ssl/key.pem"),
            implicit_tls: false,
            implicit_tls_port: 990,
        }
    }
}

impl TlsConfig {
    /// Vérifie si la configuration TLS est valide
    pub fn validate(&self) -> Result<(), TlsError> {
        if self.enabled {
            if !self.cert_file.exists() {
                return Err(TlsError::CertificateLoadError(format!(
                    "Certificate file not found: {:?}",
                    self.cert_file
                )));
            }

            if !self.key_file.exists() {
                return Err(TlsError::PrivateKeyLoadError(format!(
                    "Private key file not found: {:?}",
                    self.key_file
                )));
            }
        }

        Ok(())
    }
}
