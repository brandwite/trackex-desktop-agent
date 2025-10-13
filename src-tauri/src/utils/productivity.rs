use serde::{Deserialize, Serialize};
use regex::Regex;
use lazy_static::lazy_static;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ProductivityCategory {
    PRODUCTIVE,
    NEUTRAL,
    UNPRODUCTIVE,
}

impl Default for ProductivityCategory {
    fn default() -> Self {
        ProductivityCategory::NEUTRAL
    }
}

impl std::fmt::Display for ProductivityCategory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ProductivityCategory::PRODUCTIVE => write!(f, "PRODUCTIVE"),
            ProductivityCategory::NEUTRAL => write!(f, "NEUTRAL"),
            ProductivityCategory::UNPRODUCTIVE => write!(f, "UNPRODUCTIVE"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppRule {
    pub matcher_type: String, // EXACT, GLOB, REGEX, DOMAIN
    pub value: String,
    pub category: ProductivityCategory,
    pub priority: i32,
    pub is_active: bool,
}

#[derive(Debug, Clone)]
pub struct ProductivityClassifier {
    rules: Vec<AppRule>,
    default_category: ProductivityCategory,
}

impl ProductivityClassifier {
    pub fn new() -> Self {
        Self {
            rules: Vec::new(),
            default_category: ProductivityCategory::NEUTRAL,
        }
    }

    pub fn with_default_rules() -> Self {
        let mut classifier = Self::new();
        classifier.add_default_rules();
        classifier
    }

    pub fn add_rule(&mut self, rule: AppRule) {
        self.rules.push(rule);
        // Sort by priority (higher priority first)
        self.rules.sort_by(|a, b| b.priority.cmp(&a.priority));
    }

    pub fn add_rules(&mut self, rules: Vec<AppRule>) {
        for rule in rules {
            self.add_rule(rule);
        }
    }

    pub fn classify_app(&self, app_name: &str, app_id: &str, window_title: Option<&str>) -> ProductivityCategory {
        for rule in &self.rules {
            if !rule.is_active {
                continue;
            }

            if self.matches_rule(&rule, app_name, app_id, window_title) {
                return rule.category.clone();
            }
        }

        self.default_category.clone()
    }

    fn matches_rule(&self, rule: &AppRule, app_name: &str, app_id: &str, window_title: Option<&str>) -> bool {
        match rule.matcher_type.as_str() {
            "EXACT" => {
                app_name.eq_ignore_ascii_case(&rule.value) || 
                app_id.eq_ignore_ascii_case(&rule.value) ||
                window_title.map_or(false, |title| title.eq_ignore_ascii_case(&rule.value))
            }
            "GLOB" => {
                self.matches_glob(&rule.value, app_name) ||
                self.matches_glob(&rule.value, app_id) ||
                window_title.map_or(false, |title| self.matches_glob(&rule.value, title))
            }
            "REGEX" => {
                self.matches_regex(&rule.value, app_name) ||
                self.matches_regex(&rule.value, app_id) ||
                window_title.map_or(false, |title| self.matches_regex(&rule.value, title))
            }
            "DOMAIN" => {
                // Extract domain from window title (for web browsers)
                if let Some(title) = window_title {
                    self.extract_domain_from_title(title)
                        .map_or(false, |domain| domain.eq_ignore_ascii_case(&rule.value))
                } else {
                    false
                }
            }
            _ => false,
        }
    }

    fn matches_glob(&self, pattern: &str, text: &str) -> bool {
        // Simple glob matching - convert glob to regex
        let regex_pattern = pattern
            .replace(".", "\\.")
            .replace("*", ".*")
            .replace("?", ".");
        
        if let Ok(regex) = Regex::new(&format!("^{}$", regex_pattern)) {
            regex.is_match(text)
        } else {
            false
        }
    }

    fn matches_regex(&self, pattern: &str, text: &str) -> bool {
        if let Ok(regex) = Regex::new(pattern) {
            regex.is_match(text)
        } else {
            false
        }
    }

    fn extract_domain_from_title(&self, title: &str) -> Option<String> {
        // Extract domain from browser window titles like "Google - Mozilla Firefox"
        // or "YouTube - Google Chrome"
        lazy_static! {
            static ref DOMAIN_REGEX: Regex = Regex::new(r"^([^-\s]+)").unwrap();
        }
        
        DOMAIN_REGEX.captures(title)
            .and_then(|captures| captures.get(1))
            .map(|match_| match_.as_str().to_lowercase())
    }

    fn add_default_rules(&mut self) {
        // Productive applications
        let productive_rules = vec![
            AppRule {
                matcher_type: "EXACT".to_string(),
                value: "code.exe".to_string(),
                category: ProductivityCategory::PRODUCTIVE,
                priority: 100,
                is_active: true,
            },
            AppRule {
                matcher_type: "EXACT".to_string(),
                value: "devenv.exe".to_string(),
                category: ProductivityCategory::PRODUCTIVE,
                priority: 100,
                is_active: true,
            },
            AppRule {
                matcher_type: "EXACT".to_string(),
                value: "notepad++.exe".to_string(),
                category: ProductivityCategory::PRODUCTIVE,
                priority: 100,
                is_active: true,
            },
            AppRule {
                matcher_type: "EXACT".to_string(),
                value: "sublime_text.exe".to_string(),
                category: ProductivityCategory::PRODUCTIVE,
                priority: 100,
                is_active: true,
            },
            AppRule {
                matcher_type: "EXACT".to_string(),
                value: "atom.exe".to_string(),
                category: ProductivityCategory::PRODUCTIVE,
                priority: 100,
                is_active: true,
            },
            AppRule {
                matcher_type: "EXACT".to_string(),
                value: "vscode.exe".to_string(),
                category: ProductivityCategory::PRODUCTIVE,
                priority: 100,
                is_active: true,
            },
            AppRule {
                matcher_type: "EXACT".to_string(),
                value: "excel.exe".to_string(),
                category: ProductivityCategory::PRODUCTIVE,
                priority: 100,
                is_active: true,
            },
            AppRule {
                matcher_type: "EXACT".to_string(),
                value: "winword.exe".to_string(),
                category: ProductivityCategory::PRODUCTIVE,
                priority: 100,
                is_active: true,
            },
            AppRule {
                matcher_type: "EXACT".to_string(),
                value: "powerpnt.exe".to_string(),
                category: ProductivityCategory::PRODUCTIVE,
                priority: 100,
                is_active: true,
            },
            AppRule {
                matcher_type: "EXACT".to_string(),
                value: "outlook.exe".to_string(),
                category: ProductivityCategory::PRODUCTIVE,
                priority: 100,
                is_active: true,
            },
            AppRule {
                matcher_type: "EXACT".to_string(),
                value: "teams.exe".to_string(),
                category: ProductivityCategory::PRODUCTIVE,
                priority: 100,
                is_active: true,
            },
            AppRule {
                matcher_type: "EXACT".to_string(),
                value: "slack.exe".to_string(),
                category: ProductivityCategory::PRODUCTIVE,
                priority: 100,
                is_active: true,
            },
            AppRule {
                matcher_type: "EXACT".to_string(),
                value: "discord.exe".to_string(),
                category: ProductivityCategory::PRODUCTIVE,
                priority: 100,
                is_active: true,
            },
            AppRule {
                matcher_type: "EXACT".to_string(),
                value: "zoom.exe".to_string(),
                category: ProductivityCategory::PRODUCTIVE,
                priority: 100,
                is_active: true,
            },
            AppRule {
                matcher_type: "EXACT".to_string(),
                value: "skype.exe".to_string(),
                category: ProductivityCategory::PRODUCTIVE,
                priority: 100,
                is_active: true,
            },
            AppRule {
                matcher_type: "GLOB".to_string(),
                value: "*browser*.exe".to_string(),
                category: ProductivityCategory::NEUTRAL,
                priority: 50,
                is_active: true,
            },
            AppRule {
                matcher_type: "DOMAIN".to_string(),
                value: "github.com".to_string(),
                category: ProductivityCategory::PRODUCTIVE,
                priority: 90,
                is_active: true,
            },
            AppRule {
                matcher_type: "DOMAIN".to_string(),
                value: "stackoverflow.com".to_string(),
                category: ProductivityCategory::PRODUCTIVE,
                priority: 90,
                is_active: true,
            },
            AppRule {
                matcher_type: "DOMAIN".to_string(),
                value: "docs.microsoft.com".to_string(),
                category: ProductivityCategory::PRODUCTIVE,
                priority: 90,
                is_active: true,
            },
            AppRule {
                matcher_type: "DOMAIN".to_string(),
                value: "developer.mozilla.org".to_string(),
                category: ProductivityCategory::PRODUCTIVE,
                priority: 90,
                is_active: true,
            },
        ];

        // Unproductive applications
        let unproductive_rules = vec![
            AppRule {
                matcher_type: "EXACT".to_string(),
                value: "steam.exe".to_string(),
                category: ProductivityCategory::UNPRODUCTIVE,
                priority: 100,
                is_active: true,
            },
            AppRule {
                matcher_type: "EXACT".to_string(),
                value: "epicgameslauncher.exe".to_string(),
                category: ProductivityCategory::UNPRODUCTIVE,
                priority: 100,
                is_active: true,
            },
            AppRule {
                matcher_type: "EXACT".to_string(),
                value: "battle.net.exe".to_string(),
                category: ProductivityCategory::UNPRODUCTIVE,
                priority: 100,
                is_active: true,
            },
            AppRule {
                matcher_type: "EXACT".to_string(),
                value: "origin.exe".to_string(),
                category: ProductivityCategory::UNPRODUCTIVE,
                priority: 100,
                is_active: true,
            },
            AppRule {
                matcher_type: "EXACT".to_string(),
                value: "uplay.exe".to_string(),
                category: ProductivityCategory::UNPRODUCTIVE,
                priority: 100,
                is_active: true,
            },
            AppRule {
                matcher_type: "EXACT".to_string(),
                value: "netflix.exe".to_string(),
                category: ProductivityCategory::UNPRODUCTIVE,
                priority: 100,
                is_active: true,
            },
            AppRule {
                matcher_type: "EXACT".to_string(),
                value: "spotify.exe".to_string(),
                category: ProductivityCategory::UNPRODUCTIVE,
                priority: 100,
                is_active: true,
            },
            AppRule {
                matcher_type: "DOMAIN".to_string(),
                value: "youtube.com".to_string(),
                category: ProductivityCategory::UNPRODUCTIVE,
                priority: 90,
                is_active: true,
            },
            AppRule {
                matcher_type: "DOMAIN".to_string(),
                value: "facebook.com".to_string(),
                category: ProductivityCategory::UNPRODUCTIVE,
                priority: 90,
                is_active: true,
            },
            AppRule {
                matcher_type: "DOMAIN".to_string(),
                value: "twitter.com".to_string(),
                category: ProductivityCategory::UNPRODUCTIVE,
                priority: 90,
                is_active: true,
            },
            AppRule {
                matcher_type: "DOMAIN".to_string(),
                value: "instagram.com".to_string(),
                category: ProductivityCategory::UNPRODUCTIVE,
                priority: 90,
                is_active: true,
            },
            AppRule {
                matcher_type: "DOMAIN".to_string(),
                value: "tiktok.com".to_string(),
                category: ProductivityCategory::UNPRODUCTIVE,
                priority: 90,
                is_active: true,
            },
            AppRule {
                matcher_type: "DOMAIN".to_string(),
                value: "reddit.com".to_string(),
                category: ProductivityCategory::UNPRODUCTIVE,
                priority: 90,
                is_active: true,
            },
            AppRule {
                matcher_type: "DOMAIN".to_string(),
                value: "netflix.com".to_string(),
                category: ProductivityCategory::UNPRODUCTIVE,
                priority: 90,
                is_active: true,
            },
            AppRule {
                matcher_type: "DOMAIN".to_string(),
                value: "twitch.tv".to_string(),
                category: ProductivityCategory::UNPRODUCTIVE,
                priority: 90,
                is_active: true,
            },
        ];

        self.add_rules(productive_rules);
        self.add_rules(unproductive_rules);
    }

    pub fn get_rules(&self) -> &Vec<AppRule> {
        &self.rules
    }

    pub fn clear_rules(&mut self) {
        self.rules.clear();
    }

    #[allow(dead_code)]
    pub fn set_default_category(&mut self, category: ProductivityCategory) {
        self.default_category = category;
    }
}

impl Default for ProductivityClassifier {
    fn default() -> Self {
        Self::with_default_rules()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_exact_match() {
        let classifier = ProductivityClassifier::with_default_rules();
        
        let category = classifier.classify_app("code.exe", "C:\\Program Files\\Microsoft VS Code\\Code.exe", None);
        assert_eq!(category, ProductivityCategory::PRODUCTIVE);
        
        let category = classifier.classify_app("steam.exe", "C:\\Program Files (x86)\\Steam\\steam.exe", None);
        assert_eq!(category, ProductivityCategory::UNPRODUCTIVE);
    }

    #[test]
    fn test_glob_match() {
        let mut classifier = ProductivityClassifier::new();
        classifier.add_rule(AppRule {
            matcher_type: "GLOB".to_string(),
            value: "*browser*.exe".to_string(),
            category: ProductivityCategory::NEUTRAL,
            priority: 50,
            is_active: true,
        });
        
        let category = classifier.classify_app("chrome.exe", "C:\\Program Files\\Google\\Chrome\\Application\\chrome.exe", None);
        assert_eq!(category, ProductivityCategory::NEUTRAL);
    }

    #[test]
    fn test_domain_match() {
        let classifier = ProductivityClassifier::with_default_rules();
        
        let category = classifier.classify_app("chrome.exe", "chrome.exe", Some("GitHub - Google Chrome"));
        assert_eq!(category, ProductivityCategory::PRODUCTIVE);
        
        let category = classifier.classify_app("chrome.exe", "chrome.exe", Some("YouTube - Google Chrome"));
        assert_eq!(category, ProductivityCategory::UNPRODUCTIVE);
    }

    #[test]
    fn test_priority_order() {
        let mut classifier = ProductivityClassifier::new();
        
        // Add lower priority rule first
        classifier.add_rule(AppRule {
            matcher_type: "EXACT".to_string(),
            value: "chrome.exe".to_string(),
            category: ProductivityCategory::NEUTRAL,
            priority: 50,
            is_active: true,
        });
        
        // Add higher priority rule
        classifier.add_rule(AppRule {
            matcher_type: "EXACT".to_string(),
            value: "chrome.exe".to_string(),
            category: ProductivityCategory::PRODUCTIVE,
            priority: 100,
            is_active: true,
        });
        
        let category = classifier.classify_app("chrome.exe", "chrome.exe", None);
        assert_eq!(category, ProductivityCategory::PRODUCTIVE);
    }
}
