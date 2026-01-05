// Gestion des ratios - inspiré de glFTPd

use crate::core_quota::error::QuotaError;
use serde::{Deserialize, Serialize};

/// Représente un ratio upload:download - compatible avec la syntaxe glFTPd
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct UserRatio {
    /// Nom d'utilisateur
    pub username: String,

    /// Ratio upload (partie gauche du ratio)
    pub upload_ratio: u32,

    /// Ratio download (partie droite du ratio)
    pub download_ratio: u32,

    /// Octets uploadés
    pub uploaded_bytes: u64,

    /// Octets téléchargés
    pub downloaded_bytes: u64,

    /// Indicateur si le ratio est illimité
    pub is_unlimited: bool,
}

impl UserRatio {
    /// Crée un nouveau ratio pour un utilisateur
    /// ratio_str est au format "upload:download" (ex: "1:1", "1:2")
    pub fn new(username: &str, ratio_str: &str) -> Result<Self, QuotaError> {
        if ratio_str.to_lowercase() == "unlimited" || ratio_str == "0:0" {
            return Ok(Self::unlimited(username));
        }

        let parts: Vec<&str> = ratio_str.split(':').collect();
        if parts.len() != 2 {
            return Err(QuotaError::InvalidRatioConfig(format!(
                "Invalid ratio format: {}",
                ratio_str
            )));
        }

        let upload_ratio = parts[0].parse::<u32>().map_err(|_| {
            QuotaError::InvalidRatioConfig(format!("Invalid upload ratio: {}", parts[0]))
        })?;

        let download_ratio = parts[1].parse::<u32>().map_err(|_| {
            QuotaError::InvalidRatioConfig(format!("Invalid download ratio: {}", parts[1]))
        })?;

        Ok(Self {
            username: username.to_string(),
            upload_ratio,
            download_ratio,
            uploaded_bytes: 0,
            downloaded_bytes: 0,
            is_unlimited: false,
        })
    }

    /// Crée un ratio illimité
    pub fn unlimited(username: &str) -> Self {
        Self {
            username: username.to_string(),
            upload_ratio: 0,
            download_ratio: 0,
            uploaded_bytes: 0,
            downloaded_bytes: 0,
            is_unlimited: true,
        }
    }

    /// Vérifie si le ratio permet un téléchargement
    /// Retourne le crédit disponible en octets
    pub fn check_download(&self, download_bytes: u64) -> Result<u64, QuotaError> {
        if self.is_unlimited {
            return Ok(download_bytes);
        }

        // Calcul du crédit disponible selon la formule glFTPd:
        // crédit = (uploaded_bytes / upload_ratio) - downloaded_bytes
        let available_credit = (self.uploaded_bytes as f64 / self.upload_ratio as f64) as u64;

        if available_credit >= self.downloaded_bytes + download_bytes {
            Ok(download_bytes)
        } else {
            let remaining = available_credit.saturating_sub(self.downloaded_bytes);
            Err(QuotaError::RatioLimitReached(format!(
                "{}: Need {} more upload credit for {} download",
                self.username,
                download_bytes.saturating_sub(remaining),
                download_bytes
            )))
        }
    }

    /// Met à jour les octets téléchargés
    pub fn update_downloaded(&mut self, bytes: u64) -> Result<(), QuotaError> {
        self.check_download(bytes)?;
        self.downloaded_bytes += bytes;
        Ok(())
    }

    /// Met à jour les octets uploadés
    pub fn update_uploaded(&mut self, bytes: u64) {
        self.uploaded_bytes += bytes;
    }

    /// Retourne le ratio actuel sous forme de chaîne
    pub fn current_ratio_string(&self) -> String {
        if self.is_unlimited {
            "Unlimited".to_string()
        } else if self.downloaded_bytes == 0 {
            "No downloads yet".to_string()
        } else {
            let ratio = self.uploaded_bytes as f64 / self.downloaded_bytes as f64;
            format!("{:.2}:1", ratio)
        }
    }

    /// Retourne le ratio configuré sous forme de chaîne
    pub fn configured_ratio_string(&self) -> String {
        if self.is_unlimited {
            "Unlimited".to_string()
        } else {
            format!("{}:{}", self.upload_ratio, self.download_ratio)
        }
    }

    /// Formate le ratio pour l'affichage (comme glFTPd)
    pub fn format_ratio(&self) -> String {
        if self.is_unlimited {
            "Unlimited".to_string()
        } else {
            let upload_mb = self.uploaded_bytes as f64 / (1024.0 * 1024.0);
            let download_mb = self.downloaded_bytes as f64 / (1024.0 * 1024.0);
            let current_ratio = if self.downloaded_bytes > 0 {
                self.uploaded_bytes as f64 / self.downloaded_bytes as f64
            } else {
                0.0
            };

            format!(
                "Upload: {:.2}MB, Download: {:.2}MB, Ratio: {:.2}:1 (Configured: {}:{})",
                upload_mb, download_mb, current_ratio, self.upload_ratio, self.download_ratio
            )
        }
    }
}

/// Représente un ratio de groupe
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct GroupRatio {
    /// Nom du groupe
    pub groupname: String,

    /// Ratio upload
    pub upload_ratio: u32,

    /// Ratio download
    pub download_ratio: u32,

    /// Indicateur si le ratio est illimité
    pub is_unlimited: bool,
}

impl GroupRatio {
    pub fn new(groupname: &str, ratio_str: &str) -> Result<Self, QuotaError> {
        if ratio_str.to_lowercase() == "unlimited" || ratio_str == "0:0" {
            return Ok(Self::unlimited(groupname));
        }

        let parts: Vec<&str> = ratio_str.split(':').collect();
        if parts.len() != 2 {
            return Err(QuotaError::InvalidRatioConfig(format!(
                "Invalid ratio format: {}",
                ratio_str
            )));
        }

        let upload_ratio = parts[0].parse::<u32>().map_err(|_| {
            QuotaError::InvalidRatioConfig(format!("Invalid upload ratio: {}", parts[0]))
        })?;

        let download_ratio = parts[1].parse::<u32>().map_err(|_| {
            QuotaError::InvalidRatioConfig(format!("Invalid download ratio: {}", parts[1]))
        })?;

        Ok(Self {
            groupname: groupname.to_string(),
            upload_ratio,
            download_ratio,
            is_unlimited: false,
        })
    }

    pub fn unlimited(groupname: &str) -> Self {
        Self {
            groupname: groupname.to_string(),
            upload_ratio: 0,
            download_ratio: 0,
            is_unlimited: true,
        }
    }
}
