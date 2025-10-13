use regex::Regex;

// Domain-only mode for browsers
#[allow(dead_code)]
pub fn should_use_domain_only(bundle_id_or_process: &str) -> bool {
    #[cfg(target_os = "macos")]
    {
        let browser_bundle_ids = [
            "com.apple.Safari",
            "com.google.Chrome",
            "com.mozilla.firefox",
            "com.microsoft.Edge",
            "org.mozilla.firefox",
            "com.brave.Browser",
            "com.operasoftware.Opera",
        ];

        browser_bundle_ids
            .iter()
            .any(|&id| bundle_id_or_process.starts_with(id))
    }

    #[cfg(target_os = "windows")]
    {
        let browser_process_names = [
            "chrome.exe",
            "msedge.exe",
            "firefox.exe",
            "brave.exe",
            "opera.exe",
            "iexplore.exe", // legacy IE
        ];

        browser_process_names
            .iter()
            .any(|&name| bundle_id_or_process.eq_ignore_ascii_case(name))
    }

    #[cfg(not(any(target_os = "macos", target_os = "windows")))]
    {
        // Fallback: simple check for common browser process names
        let browser_process_names = [
            "chrome",
            "firefox",
            "brave",
            "opera",
            "edge",
            "safari",
        ];

        browser_process_names
            .iter()
            .any(|&name| bundle_id_or_process.to_lowercase().contains(name))
    }
}

// Extract domain from browser window title
#[allow(dead_code)]
pub fn extract_domain_from_title(title: &str) -> Option<String> {
    // Simple regex to extract domain from common browser title formats
    let domain_patterns = [
        r"^(.+?) [-—] .+$",  // "Domain - Browser"
        r"^(.+?) \| .+$",    // "Domain | Page Title"
        r"^(.+?) — .+$",     // "Domain — Page Title"
        r"https?://([^/\s]+)", // Direct URL in title
    ];

    for pattern in &domain_patterns {
        if let Ok(regex) = Regex::new(pattern) {
            if let Some(captures) = regex.captures(title) {
                if let Some(domain) = captures.get(1) {
                    let domain_str = domain.as_str().trim();
                    
                    // Clean up common prefixes
                    let clean_domain = domain_str
                        .strip_prefix("www.")
                        .unwrap_or(domain_str)
                        .strip_prefix("http://")
                        .unwrap_or(domain_str)
                        .strip_prefix("https://")
                        .unwrap_or(domain_str);
                    
                    return Some(clean_domain.to_string());
                }
            }
        }
    }

    None
}

// Title redaction using regex allowlist
#[allow(dead_code)]
pub fn redact_window_title(title: &str, allowlist_patterns: &[String]) -> String {
    // If no patterns provided, redact everything except domains
    if allowlist_patterns.is_empty() {
        if let Some(domain) = extract_domain_from_title(title) {
            return domain;
        }
        return "[Redacted]".to_string();
    }

    // Check if title matches any allowlist pattern
    for pattern in allowlist_patterns {
        if let Ok(regex) = Regex::new(pattern) {
            if regex.is_match(title) {
                return title.to_string(); // Allow full title
            }
        }
    }

    // Try to extract domain as fallback
    if let Some(domain) = extract_domain_from_title(title) {
        domain
    } else {
        "[Redacted]".to_string()
    }
}

// Get default title allowlist patterns
#[allow(dead_code)]
pub fn get_default_allowlist_patterns() -> Vec<String> {
    vec![
        r"^[a-zA-Z0-9\-\.]+\.[a-zA-Z]{2,}".to_string(), // Domain names
        r"^(Dashboard|Settings|Profile|Home|Login)".to_string(), // Common safe terms
        r"^\w+\s*-\s*\w+".to_string(), // Simple app names
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_domain_extraction() {
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
    }

    #[test]
    fn test_browser_detection() {
        assert!(should_use_domain_only("com.google.Chrome"));
        assert!(should_use_domain_only("com.apple.Safari"));
        assert!(!should_use_domain_only("com.apple.TextEdit"));
    }

    #[test]
    fn test_title_redaction() {
        let patterns = vec!["^Dashboard".to_string()];
        
        assert_eq!(
            redact_window_title("Dashboard - TrackEx", &patterns),
            "Dashboard - TrackEx"
        );
        
        assert_eq!(
            redact_window_title("Secret Document.docx", &patterns),
            "[Redacted]"
        );
    }
}

