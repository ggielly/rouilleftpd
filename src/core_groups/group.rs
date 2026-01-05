// Structure de groupe pour rouilleftpd
// Inspiré du système de groupes de glFTPd

use serde::{Deserialize, Serialize};
use std::collections::HashSet;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Group {
    /// Nom du groupe
    pub name: String,

    /// Description du groupe
    pub description: String,

    /// Liste des utilisateurs dans ce groupe
    pub users: HashSet<String>,

    /// Quota du groupe (en octets, 0 = illimité)
    pub quota: u64,

    /// Ratio du groupe (format "upload:download")
    pub ratio: String,

    /// Permissions du groupe
    pub permissions: GroupPermissions,
}

impl Group {
    pub fn new(name: &str, description: &str) -> Self {
        Self {
            name: name.to_string(),
            description: description.to_string(),
            users: HashSet::new(),
            quota: 0, // 0 = illimité
            ratio: "1:1".to_string(),
            permissions: GroupPermissions::default(),
        }
    }

    /// Ajoute un utilisateur au groupe
    pub fn add_user(&mut self, username: &str) -> bool {
        self.users.insert(username.to_string())
    }

    /// Supprime un utilisateur du groupe
    pub fn remove_user(&mut self, username: &str) -> bool {
        self.users.remove(username)
    }

    /// Vérifie si un utilisateur est dans le groupe
    pub fn has_user(&self, username: &str) -> bool {
        self.users.contains(username)
    }

    /// Définit le quota du groupe
    pub fn set_quota(&mut self, quota: u64) {
        self.quota = quota;
    }

    /// Définit le ratio du groupe
    pub fn set_ratio(&mut self, ratio: &str) {
        self.ratio = ratio.to_string();
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroupPermissions {
    /// Permission de lire les fichiers
    pub can_read: bool,

    /// Permission d'écrire les fichiers
    pub can_write: bool,

    /// Permission de créer des répertoires
    pub can_mkdir: bool,

    /// Permission de supprimer des fichiers
    pub can_delete: bool,

    /// Permission de renommer des fichiers
    pub can_rename: bool,

    /// Permission de lister les fichiers
    pub can_list: bool,

    /// Permission d'uploader des fichiers
    pub can_upload: bool,

    /// Permission de télécharger des fichiers
    pub can_download: bool,
}

impl Default for GroupPermissions {
    fn default() -> Self {
        Self {
            can_read: true,
            can_write: true,
            can_mkdir: true,
            can_delete: true,
            can_rename: true,
            can_list: true,
            can_upload: true,
            can_download: true,
        }
    }
}

impl GroupPermissions {
    /// Crée des permissions personnalisées
    pub fn new(
        can_read: bool,
        can_write: bool,
        can_mkdir: bool,
        can_delete: bool,
        can_rename: bool,
        can_list: bool,
        can_upload: bool,
        can_download: bool,
    ) -> Self {
        Self {
            can_read,
            can_write,
            can_mkdir,
            can_delete,
            can_rename,
            can_list,
            can_upload,
            can_download,
        }
    }

    /// Vérifie si une permission spécifique est accordée
    pub fn has_permission(&self, permission: &GroupPermission) -> bool {
        match permission {
            GroupPermission::Read => self.can_read,
            GroupPermission::Write => self.can_write,
            GroupPermission::Mkdir => self.can_mkdir,
            GroupPermission::Delete => self.can_delete,
            GroupPermission::Rename => self.can_rename,
            GroupPermission::List => self.can_list,
            GroupPermission::Upload => self.can_upload,
            GroupPermission::Download => self.can_download,
        }
    }

    /// Définit toutes les permissions
    pub fn set_all_permissions(&mut self, enabled: bool) {
        self.can_read = enabled;
        self.can_write = enabled;
        self.can_mkdir = enabled;
        self.can_delete = enabled;
        self.can_rename = enabled;
        self.can_list = enabled;
        self.can_upload = enabled;
        self.can_download = enabled;
    }
}

#[derive(Debug, Clone, Copy)]
pub enum GroupPermission {
    Read,
    Write,
    Mkdir,
    Delete,
    Rename,
    List,
    Upload,
    Download,
}
