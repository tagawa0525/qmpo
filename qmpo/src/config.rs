//! Configuration file handling for qmpo.
//!
//! Reads `config.toml` from the platform-specific data directory:
//! - Windows: `%LOCALAPPDATA%\qmpo\config.toml`
//! - macOS: `~/Library/Application Support/qmpo/config.toml`
//! - Linux: `~/.local/share/qmpo/config.toml`

use std::fs;
use std::path::{Path, PathBuf};

use directories::BaseDirs;
use serde::Deserialize;

use crate::log;

/// Top-level configuration.
#[derive(Debug, Default, Deserialize, PartialEq, Eq)]
#[serde(default)]
pub struct Config {
    pub security: SecurityConfig,
}

/// Security-related settings.
#[derive(Debug, Default, Deserialize, PartialEq, Eq)]
#[serde(default)]
pub struct SecurityConfig {
    /// Server hostnames or IP addresses that are always allowed for UNC paths,
    /// even when they resolve to non-private IPs.
    pub allowed_servers: Vec<String>,
}

impl Config {
    /// Load configuration from the default config file path.
    /// Returns `Config::default()` if the file does not exist or cannot be parsed.
    pub fn load() -> Self {
        Self::config_path()
            .and_then(|p| Self::load_from(&p))
            .unwrap_or_default()
    }

    /// Load configuration from a specific file path.
    /// Returns `None` if the file does not exist or cannot be parsed.
    fn load_from(path: &Path) -> Option<Self> {
        let content = fs::read_to_string(path).ok()?;
        match Self::parse(&content) {
            Some(config) => Some(config),
            None => {
                log::error(&format!(
                    "Failed to parse config file: {}",
                    path.display()
                ));
                None
            }
        }
    }

    /// Parse a TOML string into a Config.
    fn parse(toml_str: &str) -> Option<Self> {
        toml::from_str(toml_str).ok()
    }

    /// Returns the platform-specific path to `config.toml`.
    fn config_path() -> Option<PathBuf> {
        Some(BaseDirs::new()?.data_local_dir().join("qmpo").join("config.toml"))
    }
}

impl SecurityConfig {
    /// Check if a server hostname or IP is in the allowed list.
    /// Comparison is case-insensitive.
    pub fn is_server_allowed(&self, server: &str) -> bool {
        self.allowed_servers
            .iter()
            .any(|s| s.eq_ignore_ascii_case(server))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_full_config() {
        let toml = r#"
[security]
allowed_servers = ["fileserver.example.com", "203.0.113.50"]
"#;
        let config = Config::parse(toml).unwrap();
        assert_eq!(
            config.security.allowed_servers,
            vec!["fileserver.example.com", "203.0.113.50"]
        );
    }

    #[test]
    fn test_parse_empty() {
        let config = Config::parse("").unwrap();
        assert_eq!(config, Config::default());
    }

    #[test]
    fn test_parse_no_security_section() {
        let toml = r#"
[other]
key = "value"
"#;
        let config = Config::parse(toml).unwrap();
        assert!(config.security.allowed_servers.is_empty());
    }

    #[test]
    fn test_parse_empty_allowed_servers() {
        let toml = r#"
[security]
allowed_servers = []
"#;
        let config = Config::parse(toml).unwrap();
        assert!(config.security.allowed_servers.is_empty());
    }

    #[test]
    fn test_is_server_allowed() {
        let config = SecurityConfig {
            allowed_servers: vec!["FileServer.Example.COM".to_string()],
        };
        assert!(config.is_server_allowed("fileserver.example.com"));
        assert!(config.is_server_allowed("FILESERVER.EXAMPLE.COM"));
        assert!(!config.is_server_allowed("other.example.com"));
    }

    #[test]
    fn test_is_server_allowed_ip() {
        let config = SecurityConfig {
            allowed_servers: vec!["203.0.113.50".to_string()],
        };
        assert!(config.is_server_allowed("203.0.113.50"));
        assert!(!config.is_server_allowed("203.0.113.51"));
    }

    #[test]
    fn test_default_config_allows_nothing() {
        let config = Config::default();
        assert!(!config.security.is_server_allowed("anything"));
    }

    #[test]
    fn test_parse_invalid_toml() {
        let config = Config::parse("[invalid toml ===");
        assert!(config.is_none());
    }

    #[test]
    fn test_parse_wrong_type() {
        // allowed_servers should be an array, not a string
        let toml = r#"
[security]
allowed_servers = "not-an-array"
"#;
        let config = Config::parse(toml);
        assert!(config.is_none());
    }
}
