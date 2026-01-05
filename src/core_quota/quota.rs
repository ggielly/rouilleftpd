// Gestion des quotas - inspiré de glFTPd

use crate::core_quota::error::QuotaError;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Représente un quota utilisateur - compatible avec la syntaxe glFTPd
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct UserQuota {
    /// Nom d'utilisateur
    pub username: String,

    /// Quota en octets (0 = illimité)
    pub max_bytes: u64,

    /// Octets actuellement utilisés
    pub used_bytes: u64,

    /// Répertoire de base pour le calcul du quota
    pub base_dir: PathBuf,

    /// Indicateur si le quota est illimité
    pub is_unlimited: bool,
}

impl UserQuota {
    /// Crée un nouveau quota pour un utilisateur
    pub fn new(username: &str, max_bytes: u64, base_dir: PathBuf) -> Self {
        let is_unlimited = max_bytes == 0;
        Self {
            username: username.to_string(),
            max_bytes,
            used_bytes: 0,
            base_dir,
            is_unlimited,
        }
    }

    /// Crée un quota illimité
    pub fn unlimited(username: &str, base_dir: PathBuf) -> Self {
        Self {
            username: username.to_string(),
            max_bytes: 0,
            used_bytes: 0,
            base_dir,
            is_unlimited: true,
        }
    }

    /// Vérifie si le quota est dépassé
    pub fn check_quota(&self, additional_bytes: u64) -> Result<(), QuotaError> {
        if self.is_unlimited {
            return Ok(());
        }

        if self.used_bytes + additional_bytes > self.max_bytes {
            Err(QuotaError::QuotaExceeded(self.username.clone()))
        } else {
            Ok(())
        }
    }

    /// Met à jour les octets utilisés
    pub fn update_used_bytes(&mut self, bytes: u64) -> Result<(), QuotaError> {
        self.check_quota(bytes)?;
        self.used_bytes += bytes;
        Ok(())
    }

    /// Réduit les octets utilisés (pour les suppressions)
    pub fn reduce_used_bytes(&mut self, bytes: u64) {
        if self.used_bytes >= bytes {
            self.used_bytes -= bytes;
        } else {
            self.used_bytes = 0;
        }
    }

    /// Retourne le pourcentage d'utilisation
    pub fn usage_percentage(&self) -> f64 {
        if self.is_unlimited || self.max_bytes == 0 {
            0.0
        } else {
            (self.used_bytes as f64 / self.max_bytes as f64) * 100.0
        }
    }

    /// Formate le quota pour l'affichage (comme glFTPd)
    pub fn format_quota(&self) -> String {
        if self.is_unlimited {
            "Unlimited".to_string()
        } else {
            let used_mb = self.used_bytes as f64 / (1024.0 * 1024.0);
            let max_mb = self.max_bytes as f64 / (1024.0 * 1024.0);
            format!(
                "{:.2}MB / {:.2}MB ({:.1}%)",
                used_mb,
                max_mb,
                self.usage_percentage()
            )
        }
    }
}

/// Représente un quota de groupe
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct GroupQuota {
    /// Nom du groupe
    pub groupname: String,

    /// Quota en octets (0 = illimité)
    pub max_bytes: u64,

    /// Indicateur si le quota est illimité
    pub is_unlimited: bool,
}

impl GroupQuota {
    pub fn new(groupname: &str, max_bytes: u64) -> Self {
        let is_unlimited = max_bytes == 0;
        Self {
            groupname: groupname.to_string(),
            max_bytes,
            is_unlimited,
        }
    }

    pub fn unlimited(groupname: &str) -> Self {
        Self {
            groupname: groupname.to_string(),
            max_bytes: 0,
            is_unlimited: true,
        }
    }
}
