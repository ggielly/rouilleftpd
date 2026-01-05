// Gestionnaire de groupes pour rouilleftpd
// Inspiré du système de groupes de glFTPd

use crate::core_groups::{error::GroupError, group::Group};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

pub struct GroupManager {
    groups: Arc<Mutex<HashMap<String, Group>>>, // Groups par nom
    storage_file: PathBuf,
}

impl GroupManager {
    pub fn new(storage_file: PathBuf) -> Self {
        Self {
            groups: Arc::new(Mutex::new(HashMap::new())),
            storage_file,
        }
    }

    /// Charge les groupes depuis le fichier
    pub fn load(&self) -> Result<(), GroupError> {
        if !self.storage_file.exists() {
            // Créer des groupes par défaut si le fichier n'existe pas
            self.create_default_groups()?;
            return Ok(());
        }

        let content = std::fs::read_to_string(&self.storage_file)
            .map_err(|e| GroupError::GroupReadError(e.to_string()))?;

        let groups: HashMap<String, Group> = serde_json::from_str(&content)
            .map_err(|e| GroupError::GroupReadError(e.to_string()))?;

        *self.groups.lock().unwrap() = groups;

        Ok(())
    }

    /// Sauvegarde les groupes dans le fichier
    pub fn save(&self) -> Result<(), GroupError> {
        let groups = self.groups.lock().unwrap();
        let content = serde_json::to_string(&*groups)
            .map_err(|e| GroupError::GroupWriteError(e.to_string()))?;

        std::fs::write(&self.storage_file, content)
            .map_err(|e| GroupError::GroupWriteError(e.to_string()))?;

        Ok(())
    }

    /// Crée des groupes par défaut
    fn create_default_groups(&self) -> Result<(), GroupError> {
        let mut groups = self.groups.lock().unwrap();

        // Groupe admin
        let mut admin_group = Group::new("admins", "Administrators");
        admin_group.set_quota(0); // Illimité
        admin_group.set_ratio("0:0"); // Illimité
        groups.insert("admins".to_string(), admin_group);

        // Groupe utilisateurs
        let mut users_group = Group::new("users", "Regular Users");
        users_group.set_quota(10737418240); // 10 GB
        users_group.set_ratio("1:1");
        groups.insert("users".to_string(), users_group);

        // Groupe invités
        let mut guests_group = Group::new("guests", "Guest Users");
        guests_group.set_quota(1073741824); // 1 GB
        guests_group.set_ratio("1:2");
        // Restreindre les permissions pour les invités
        guests_group.permissions.can_write = false;
        guests_group.permissions.can_mkdir = false;
        guests_group.permissions.can_delete = false;
        guests_group.permissions.can_rename = false;
        groups.insert("guests".to_string(), guests_group);

        Ok(())
    }

    /// Obtient un groupe par nom
    pub fn get_group(&self, group_name: &str) -> Option<Group> {
        let groups = self.groups.lock().unwrap();
        groups.get(group_name).cloned()
    }

    /// Crée un nouveau groupe
    pub fn create_group(&self, name: &str, description: &str) -> Result<(), GroupError> {
        let mut groups = self.groups.lock().unwrap();

        if groups.contains_key(name) {
            return Err(GroupError::InvalidGroupConfig(format!(
                "Group {} already exists",
                name
            )));
        }

        let group = Group::new(name, description);
        groups.insert(name.to_string(), group);

        Ok(())
    }

    /// Supprime un groupe
    pub fn delete_group(&self, group_name: &str) -> Result<(), GroupError> {
        let mut groups = self.groups.lock().unwrap();

        if !groups.contains_key(group_name) {
            return Err(GroupError::GroupNotFound(group_name.to_string()));
        }

        // Ne pas permettre la suppression des groupes par défaut
        if ["admins", "users", "guests"].contains(&group_name) {
            return Err(GroupError::PermissionDenied(format!(
                "Cannot delete default group: {}",
                group_name
            )));
        }

        groups.remove(group_name);

        Ok(())
    }

    /// Ajoute un utilisateur à un groupe
    pub fn add_user_to_group(&self, username: &str, group_name: &str) -> Result<(), GroupError> {
        let mut groups = self.groups.lock().unwrap();

        let group = groups
            .get_mut(group_name)
            .ok_or_else(|| GroupError::GroupNotFound(group_name.to_string()))?;

        group.add_user(username);

        Ok(())
    }

    /// Supprime un utilisateur d'un groupe
    pub fn remove_user_from_group(
        &self,
        username: &str,
        group_name: &str,
    ) -> Result<(), GroupError> {
        let mut groups = self.groups.lock().unwrap();

        let group = groups
            .get_mut(group_name)
            .ok_or_else(|| GroupError::GroupNotFound(group_name.to_string()))?;

        if !group.remove_user(username) {
            return Err(GroupError::UserNotFound(username.to_string()));
        }

        Ok(())
    }

    /// Vérifie si un utilisateur est dans un groupe
    pub fn is_user_in_group(&self, username: &str, group_name: &str) -> Result<bool, GroupError> {
        let groups = self.groups.lock().unwrap();

        let group = groups
            .get(group_name)
            .ok_or_else(|| GroupError::GroupNotFound(group_name.to_string()))?;

        Ok(group.has_user(username))
    }

    /// Obtient les groupes d'un utilisateur
    pub fn get_user_groups(&self, username: &str) -> Result<Vec<String>, GroupError> {
        let groups = self.groups.lock().unwrap();

        let user_groups = groups
            .iter()
            .filter(|(_, group)| group.has_user(username))
            .map(|(name, _)| name.clone())
            .collect();

        Ok(user_groups)
    }

    /// Définit le quota d'un groupe
    pub fn set_group_quota(&self, group_name: &str, quota: u64) -> Result<(), GroupError> {
        let mut groups = self.groups.lock().unwrap();

        let group = groups
            .get_mut(group_name)
            .ok_or_else(|| GroupError::GroupNotFound(group_name.to_string()))?;

        group.set_quota(quota);

        Ok(())
    }

    /// Définit le ratio d'un groupe
    pub fn set_group_ratio(&self, group_name: &str, ratio: &str) -> Result<(), GroupError> {
        let mut groups = self.groups.lock().unwrap();

        let group = groups
            .get_mut(group_name)
            .ok_or_else(|| GroupError::GroupNotFound(group_name.to_string()))?;

        group.set_ratio(ratio);

        Ok(())
    }

    /// Vérifie les permissions d'un utilisateur
    pub fn check_permission(
        &self,
        username: &str,
        group_name: &str,
        permission: &crate::core_groups::group::GroupPermission,
    ) -> Result<bool, GroupError> {
        let groups = self.groups.lock().unwrap();

        let group = groups
            .get(group_name)
            .ok_or_else(|| GroupError::GroupNotFound(group_name.to_string()))?;

        if !group.has_user(username) {
            return Err(GroupError::UserNotFound(username.to_string()));
        }

        Ok(group.permissions.has_permission(permission))
    }
}
