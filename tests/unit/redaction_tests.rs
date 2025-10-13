#[cfg(test)]
mod redaction_tests {
    use trackex_agent_lib::policy::privacy::{
        extract_domain_from_title, 
        redact_window_title, 
        should_use_domain_only,
        get_default_allowlist_patterns
    };

    #[test]
    fn test_browser_detection() {
        // Should detect browsers
        assert!(should_use_domain_only("com.google.Chrome"));
        assert!(should_use_domain_only("com.apple.Safari"));
        assert!(should_use_domain_only("com.mozilla.firefox"));
        assert!(should_use_domain_only("com.microsoft.Edge"));
        assert!(should_use_domain_only("com.brave.Browser"));
        assert!(should_use_domain_only("com.operasoftware.Opera"));
        
        // Should not detect non-browsers
        assert!(!should_use_domain_only("com.apple.TextEdit"));
        assert!(!should_use_domain_only("com.microsoft.Word"));
        assert!(!should_use_domain_only("com.adobe.Photoshop"));
    }

    #[test]
    fn test_domain_extraction() {
        // Test various browser title formats
        assert_eq!(
            extract_domain_from_title("Google - Google Chrome"),
            Some("Google".to_string())
        );
        
        assert_eq!(
            extract_domain_from_title("github.com | GitHub"),
            Some("github.com".to_string())
        );
        
        assert_eq!(
            extract_domain_from_title("Stack Overflow — Where Developers Learn"),
            Some("Stack Overflow".to_string())
        );
        
        assert_eq!(
            extract_domain_from_title("https://www.example.com/page"),
            Some("www.example.com".to_string())
        );
        
        // Test with no recognizable pattern
        assert_eq!(
            extract_domain_from_title("Random Window Title"),
            None
        );
    }

    #[test]
    fn test_title_redaction_with_patterns() {
        let patterns = vec![
            "^Dashboard".to_string(),
            "TrackEx".to_string(),
            r"^[a-zA-Z0-9\-\.]+\.[a-zA-Z]{2,}".to_string(), // Domain pattern
        ];
        
        // Should allow titles matching patterns
        assert_eq!(
            redact_window_title("Dashboard - TrackEx", &patterns),
            "Dashboard - TrackEx"
        );
        
        assert_eq!(
            redact_window_title("TrackEx Settings", &patterns),
            "TrackEx Settings"
        );
        
        assert_eq!(
            redact_window_title("google.com", &patterns),
            "google.com"
        );
        
        // Should redact titles not matching patterns
        assert_eq!(
            redact_window_title("Secret Document.docx", &patterns),
            "[Redacted]"
        );
        
        assert_eq!(
            redact_window_title("Private Email", &patterns),
            "[Redacted]"
        );
    }

    #[test]
    fn test_title_redaction_domain_fallback() {
        let empty_patterns: Vec<String> = vec![];
        
        // Should extract domain when no patterns provided
        assert_eq!(
            redact_window_title("Google - Chrome", &empty_patterns),
            "Google"
        );
        
        assert_eq!(
            redact_window_title("github.com | Code Repository", &empty_patterns),
            "github.com"
        );
        
        // Should redact when no domain extractable
        assert_eq!(
            redact_window_title("Random Document", &empty_patterns),
            "[Redacted]"
        );
    }

    #[test]
    fn test_default_allowlist_patterns() {
        let patterns = get_default_allowlist_patterns();
        assert!(!patterns.is_empty());
        
        // Test that default patterns work
        assert_eq!(
            redact_window_title("Dashboard", &patterns),
            "Dashboard"
        );
        
        assert_eq!(
            redact_window_title("google.com", &patterns),
            "google.com"
        );
    }

    #[test]
    fn test_complex_browser_titles() {
        let empty_patterns: Vec<String> = vec![];
        
        // Test complex real-world browser titles
        let test_cases = vec![
            ("(3) Facebook", Some("Facebook".to_string())),
            ("Gmail - john@example.com", Some("Gmail".to_string())),
            ("YouTube — Broadcast Yourself", Some("YouTube".to_string())),
            ("New Tab - Google Chrome", Some("New Tab".to_string())),
            ("https://trackex.com/dashboard", Some("trackex.com".to_string())),
        ];
        
        for (title, expected) in test_cases {
            let result = extract_domain_from_title(title);
            assert_eq!(result, expected, "Failed for title: {}", title);
        }
    }

    #[test]
    fn test_regex_patterns() {
        let patterns = vec![
            r"^\d+\s+notifications?".to_string(), // Notification counts
            r"^(Settings|Preferences|Options)".to_string(), // Settings windows
            r"Meeting.*Zoom".to_string(), // Zoom meetings
        ];
        
        // Should allow matching patterns
        assert_eq!(
            redact_window_title("5 notifications", &patterns),
            "5 notifications"
        );
        
        assert_eq!(
            redact_window_title("Settings - System Preferences", &patterns),
            "Settings - System Preferences"
        );
        
        assert_eq!(
            redact_window_title("Meeting with John - Zoom", &patterns),
            "Meeting with John - Zoom"
        );
        
        // Should redact non-matching
        assert_eq!(
            redact_window_title("Private document.pdf", &patterns),
            "[Redacted]"
        );
    }
}
