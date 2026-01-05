// Module de gestion des groupes pour rouilleftpd
// Inspiré du système de groupes de glFTPd

pub mod error;
pub mod group;
pub mod group_manager;

pub use error::GroupError;
pub use group::Group;
pub use group_manager::GroupManager;
