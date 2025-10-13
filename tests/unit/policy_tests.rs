#[cfg(test)]
mod policy_tests {
    use trackex_agent_lib::policy::toggles::{PolicyConfig};

    #[test]
    fn test_default_policy_config() {
        let config = PolicyConfig::default();
        
        assert!(!config.screenshot_enabled);
        assert_eq!(config.screenshot_interval_minutes, 0);
        assert!(config.domain_only_mode);
        assert!(config.title_redaction_enabled);
        assert_eq!(config.idle_threshold_seconds, 300);
        assert!(!config.allowlist_patterns.is_empty());
    }

    #[test]
    fn test_policy_config_from_env() {
        // Set test environment variables
        std::env::set_var("TRACKEX_SCREENSHOT_ENABLED", "true");
        std::env::set_var("TRACKEX_SCREENSHOT_INTERVAL", "30");
        std::env::set_var("TRACKEX_DOMAIN_ONLY", "false");
        std::env::set_var("TRACKEX_TITLE_REDACTION", "false");
        std::env::set_var("TRACKEX_IDLE_THRESHOLD", "600");
        
        let config = PolicyConfig::from_env();
        
        assert!(config.screenshot_enabled);
        assert_eq!(config.screenshot_interval_minutes, 30);
        assert!(!config.domain_only_mode);
        assert!(!config.title_redaction_enabled);
        assert_eq!(config.idle_threshold_seconds, 600);
        
        // Clean up
        std::env::remove_var("TRACKEX_SCREENSHOT_ENABLED");
        std::env::remove_var("TRACKEX_SCREENSHOT_INTERVAL");
        std::env::remove_var("TRACKEX_DOMAIN_ONLY");
        std::env::remove_var("TRACKEX_TITLE_REDACTION");
        std::env::remove_var("TRACKEX_IDLE_THRESHOLD");
    }

    #[test]
    fn test_should_take_screenshot() {
        let mut config = PolicyConfig::default();
        
        // Disabled by default
        assert!(!config.should_take_screenshot());
        
        // Enable screenshots but no interval
        config.screenshot_enabled = true;
        assert!(!config.should_take_screenshot());
        
        // Enable with interval
        config.screenshot_interval_minutes = 15;
        assert!(config.should_take_screenshot());
    }

    #[test]
    fn test_get_screenshot_interval_seconds() {
        let mut config = PolicyConfig::default();
        
        config.screenshot_interval_minutes = 15;
        assert_eq!(config.get_screenshot_interval_seconds(), 900); // 15 * 60
        
        config.screenshot_interval_minutes = 30;
        assert_eq!(config.get_screenshot_interval_seconds(), 1800); // 30 * 60
        
        config.screenshot_interval_minutes = 60;
        assert_eq!(config.get_screenshot_interval_seconds(), 3600); // 60 * 60
    }

    #[test]
    fn test_should_redact_title() {
        let mut config = PolicyConfig::default();
        
        // Browser with domain-only mode enabled (default)
        assert!(config.should_redact_title("com.google.Chrome"));
        assert!(config.should_redact_title("com.apple.Safari"));
        
        // Non-browser with redaction enabled (default)
        assert!(config.should_redact_title("com.apple.TextEdit"));
        
        // Disable title redaction
        config.title_redaction_enabled = false;
        assert!(!config.should_redact_title("com.apple.TextEdit"));
        
        // Browser should still use domain-only if enabled
        config.domain_only_mode = true;
        assert!(config.should_redact_title("com.google.Chrome"));
        
        // Disable domain-only mode
        config.domain_only_mode = false;
        assert!(!config.should_redact_title("com.google.Chrome"));
    }

    #[test]
    fn test_policy_edge_cases() {
        let mut config = PolicyConfig::default();
        
        // Test with empty bundle ID
        assert!(config.should_redact_title(""));
        
        // Test with unknown bundle ID
        assert!(config.should_redact_title("unknown.app.bundle"));
        
        // Test screenshot interval of 0
        config.screenshot_enabled = true;
        config.screenshot_interval_minutes = 0;
        assert!(!config.should_take_screenshot());
    }

    #[test]
    fn test_invalid_env_values() {
        // Set invalid environment variables
        std::env::set_var("TRACKEX_SCREENSHOT_ENABLED", "invalid");
        std::env::set_var("TRACKEX_SCREENSHOT_INTERVAL", "not_a_number");
        std::env::set_var("TRACKEX_IDLE_THRESHOLD", "invalid");
        
        let config = PolicyConfig::from_env();
        
        // Should fall back to defaults for invalid values
        assert!(!config.screenshot_enabled); // Default false
        assert_eq!(config.screenshot_interval_minutes, 0); // Default 0
        assert_eq!(config.idle_threshold_seconds, 300); // Default 300
        
        // Clean up
        std::env::remove_var("TRACKEX_SCREENSHOT_ENABLED");
        std::env::remove_var("TRACKEX_SCREENSHOT_INTERVAL");
        std::env::remove_var("TRACKEX_IDLE_THRESHOLD");
    }
}
