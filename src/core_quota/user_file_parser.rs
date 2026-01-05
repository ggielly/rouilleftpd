// Parser pour les fichiers utilisateurs avec quota/ratio
// Inspiré du format glFTPd

use crate::core_quota::error::QuotaError;
use log::warn;
use std::collections::HashMap;
use std::path::Path;

/// Parse un fichier utilisateur et extrait les informations de quota/ratio
pub fn parse_user_file(file_path: &Path) -> Result<(Option<u64>, Option<String>), QuotaError> {
    let content = std::fs::read_to_string(file_path)
        .map_err(|e| QuotaError::QuotaReadError(e.to_string()))?;

    let mut quota = None;
    let mut ratio = None;

    for line in content.lines() {
        // Ignorer les commentaires et les lignes vides
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        // Parser les lignes au format key=value
        if let Some((key, value)) = line.split_once('=') {
            let key = key.trim().to_lowercase();
            let value = value.trim();

            match key.as_str() {
                "quota" => {
                    if value.to_lowercase() == "unlimited" {
                        quota = Some(0); // 0 signifie illimité
                    } else {
                        // Parser le quota en octets
                        // Supporter les suffixes comme GB, MB, etc.
                        quota = Some(parse_size(value)?);
                    }
                }
                "ratio" => {
                    if value.to_lowercase() == "unlimited" {
                        ratio = Some("0:0".to_string()); // 0:0 signifie illimité
                    } else {
                        // Valider le format du ratio
                        if value.contains(':') {
                            ratio = Some(value.to_string());
                        } else {
                            return Err(QuotaError::InvalidRatioConfig(format!(
                                "Invalid ratio format: {}",
                                value
                            )));
                        }
                    }
                }
                _ => {
                    // Ignorer les autres clés
                }
            }
        }
    }

    Ok((quota, ratio))
}

/// Parse une taille avec suffixe (ex: 10GB, 5MB, etc.)
fn parse_size(size_str: &str) -> Result<u64, QuotaError> {
    let size_str = size_str.trim();

    // Extraire le nombre et le suffixe
    let (num_str, suffix) = if size_str.ends_with(|c: char| c.is_ascii_alphabetic()) {
        let split_pos = size_str
            .char_indices()
            .find(|(_, c)| c.is_ascii_alphabetic())
            .map(|(i, _)| i)
            .unwrap_or(size_str.len());

        (&size_str[..split_pos], &size_str[split_pos..])
    } else {
        (size_str, "")
    };

    let num: u64 = num_str
        .parse()
        .map_err(|e| QuotaError::InvalidQuotaConfig(format!("Invalid size number: {}", e)))?;

    let multiplier = match suffix.to_uppercase().as_str() {
        "KB" | "K" => 1024,
        "MB" | "M" => 1024 * 1024,
        "GB" | "G" => 1024 * 1024 * 1024,
        "TB" | "T" => 1024 * 1024 * 1024 * 1024,
        "" => 1, // Pas de suffixe, supposer que c'est en octets
        _ => {
            return Err(QuotaError::InvalidQuotaConfig(format!(
                "Unknown size suffix: {}",
                suffix
            )))
        }
    };

    Ok(num * multiplier)
}

/// Charge les configurations de quota/ratio pour tous les utilisateurs
pub fn load_user_quota_configs(
    users_dir: &Path,
) -> Result<HashMap<String, (Option<u64>, Option<String>)>, QuotaError> {
    let mut configs = HashMap::new();

    if !users_dir.exists() {
        return Ok(configs);
    }

    for entry in
        std::fs::read_dir(users_dir).map_err(|e| QuotaError::QuotaReadError(e.to_string()))?
    {
        let entry = entry.map_err(|e| QuotaError::QuotaReadError(e.to_string()))?;
        let path = entry.path();

        if path.extension().and_then(|s| s.to_str()) == Some("user") {
            if let Some(username) = path.file_stem().and_then(|s| s.to_str()) {
                match parse_user_file(&path) {
                    Ok((quota, ratio)) => {
                        configs.insert(username.to_string(), (quota, ratio));
                    }
                    Err(e) => {
                        warn!("Failed to parse user file {}: {}", path.display(), e);
                    }
                }
            }
        }
    }

    Ok(configs)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_parse_size() {
        assert_eq!(parse_size("1024").unwrap(), 1024);
        assert_eq!(parse_size("1KB").unwrap(), 1024);
        assert_eq!(parse_size("1K").unwrap(), 1024);
        assert_eq!(parse_size("1MB").unwrap(), 1024 * 1024);
        assert_eq!(parse_size("1M").unwrap(), 1024 * 1024);
        assert_eq!(parse_size("1GB").unwrap(), 1024 * 1024 * 1024);
        assert_eq!(parse_size("1G").unwrap(), 1024 * 1024 * 1024);
    }

    #[test]
    fn test_parse_user_file() {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "username=testuser").unwrap();
        writeln!(file, "quota=10GB").unwrap();
        writeln!(file, "ratio=1:2").unwrap();

        let (quota, ratio) = parse_user_file(file.path()).unwrap();
        assert_eq!(quota, Some(10 * 1024 * 1024 * 1024));
        assert_eq!(ratio, Some("1:2".to_string()));
    }
}
