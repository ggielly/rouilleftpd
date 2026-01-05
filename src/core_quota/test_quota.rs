// Test simple pour vérifier le système de quota

#[cfg(test)]
mod tests {
    use super::super::*;
    use std::sync::Arc;
    use std::path::PathBuf;
    
    #[test]
    fn test_quota_creation() {
        let quota = UserQuota::new("testuser", 1024 * 1024, PathBuf::from("/test"));
        assert_eq!(quota.username, "testuser");
        assert_eq!(quota.max_bytes, 1024 * 1024);
        assert_eq!(quota.used_bytes, 0);
        assert!(!quota.is_unlimited);
    }
    
    #[test]
    fn test_unlimited_quota() {
        let quota = UserQuota::unlimited("admin", PathBuf::from("/admin"));
        assert_eq!(quota.username, "admin");
        assert_eq!(quota.max_bytes, 0);
        assert!(quota.is_unlimited);
    }
    
    #[test]
    fn test_quota_check() {
        let mut quota = UserQuota::new("testuser", 1024, PathBuf::from("/test"));
        
        // Should succeed
        assert!(quota.check_quota(512).is_ok());
        quota.update_used_bytes(512).unwrap();
        
        // Should still succeed
        assert!(quota.check_quota(512).is_ok());
        quota.update_used_bytes(512).unwrap();
        
        // Should fail
        assert!(quota.check_quota(1).is_err());
    }
    
    #[test]
    fn test_ratio_creation() {
        let ratio = UserRatio::new("testuser", "1:2").unwrap();
        assert_eq!(ratio.username, "testuser");
        assert_eq!(ratio.upload_ratio, 1);
        assert_eq!(ratio.download_ratio, 2);
        assert!(!ratio.is_unlimited);
    }
    
    #[test]
    fn test_unlimited_ratio() {
        let ratio = UserRatio::unlimited("admin");
        assert_eq!(ratio.username, "admin");
        assert!(ratio.is_unlimited);
    }
    
    #[test]
    fn test_ratio_check() {
        let mut ratio = UserRatio::new("testuser", "1:1").unwrap();
        
        // Upload some data
        ratio.update_uploaded(1024);
        
        // Should be able to download same amount
        assert!(ratio.check_download(1024).is_ok());
        ratio.update_downloaded(1024).unwrap();
        
        // Should not be able to download more without uploading
        assert!(ratio.check_download(1).is_err());
        
        // Upload more
        ratio.update_uploaded(1024);
        
        // Should be able to download again
        assert!(ratio.check_download(1024).is_ok());
    }
    
    #[test]
    fn test_quota_manager() {
        let config = CoreQuotaConfig {
            default_quota: 1024 * 1024,
            default_ratio: "1:1".to_string(),
            quota_storage_file: PathBuf::from("data/test_quotas.json"),
            ratio_storage_file: PathBuf::from("data/test_ratios.json"),
            stats_storage_file: PathBuf::from("data/test_stats.json"),
            enable_quota: true,
            enable_ratio: true,
        };
        
        let group_config = GroupQuotaConfig::new();
        let user_config = UserQuotaConfig::new();
        
        let manager = QuotaManager::new(config, group_config, user_config);
        
        // Test getting quota for new user
        let quota = manager.get_or_create_user_quota("newuser", PathBuf::from("/test")).unwrap();
        assert_eq!(quota.max_bytes, 1024 * 1024);
        
        // Test getting ratio for new user
        let ratio = manager.get_or_create_user_ratio("newuser").unwrap();
        assert_eq!(ratio.upload_ratio, 1);
        assert_eq!(ratio.download_ratio, 1);
    }
}