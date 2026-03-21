//! Configuration file handling for qmpo.
//!
//! Loads `config.toml` from two locations and merges them (user overrides machine):
//!
//! Machine-wide (organization):
//! - Windows: `%PROGRAMDATA%\qmpo\config.toml`
//! - macOS: `/Library/Application Support/qmpo/config.toml`
//! - Linux: `/etc/qmpo/config.toml`
//!
//! User-specific:
//! - Windows: `%LOCALAPPDATA%\qmpo\config.toml`
//! - macOS: `~/Library/Application Support/qmpo/config.toml`
//! - Linux: `~/.config/qmpo/config.toml`
//!
//! The environment variable `QMPO_CONFIG_DIR` overrides the **user** config
//! path (not the machine path), so organization policy always applies.

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

/// Intermediate struct for partial deserialization (supports merge).
#[derive(Debug, Default, Deserialize)]
#[serde(default)]
struct PartialConfig {
    security: Option<PartialSecurityConfig>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(default)]
struct PartialSecurityConfig {
    allowed_servers: Option<Vec<String>>,
}

impl Config {
    /// Load configuration by merging machine-wide and user configs.
    /// `QMPO_CONFIG_DIR` overrides the user config path (machine policy always applies).
    /// Returns `Config::default()` if no config files exist or cannot be parsed.
    pub fn load() -> Self {
        // Load machine config as base (always applies — org policy)
        let machine = machine_config_path()
            .and_then(|p| Self::load_from(&p))
            .unwrap_or_default();

        // QMPO_CONFIG_DIR overrides the user config path, not the machine config
        let user_path = if let Ok(dir) = std::env::var("QMPO_CONFIG_DIR") {
            Some(PathBuf::from(dir).join("config.toml"))
        } else {
            user_config_path()
        };

        // Load user config as overlay (using partial deserialization)
        let user_partial = user_path.and_then(|p| Self::load_partial_from(&p));

        match user_partial {
            Some(partial) => machine.merge(partial),
            None => machine,
        }
    }

    /// Load configuration from a specific file path.
    /// Returns `None` if the file does not exist or cannot be parsed.
    fn load_from(path: &Path) -> Option<Self> {
        let content = fs::read_to_string(path).ok()?;
        match Self::parse(&content) {
            Some(config) => Some(config),
            None => {
                log::error(&format!("Failed to parse config file: {}", path.display()));
                None
            }
        }
    }

    /// Load partial configuration for merging.
    fn load_partial_from(path: &Path) -> Option<PartialConfig> {
        let content = fs::read_to_string(path).ok()?;
        match toml::from_str(&content) {
            Ok(partial) => Some(partial),
            Err(_) => {
                log::error(&format!("Failed to parse config file: {}", path.display()));
                None
            }
        }
    }

    /// Parse a TOML string into a Config.
    fn parse(toml_str: &str) -> Option<Self> {
        toml::from_str(toml_str).ok()
    }

    /// Merge user partial config over self (machine config).
    /// Only fields explicitly set in the user config override the machine values.
    fn merge(mut self, user: PartialConfig) -> Self {
        if let Some(security) = user.security
            && let Some(allowed_servers) = security.allowed_servers
        {
            self.security.allowed_servers = allowed_servers;
        }
        self
    }
}

/// Returns the machine-wide config path.
fn machine_config_path() -> Option<PathBuf> {
    #[cfg(target_os = "windows")]
    {
        std::env::var("PROGRAMDATA")
            .ok()
            .map(|pd| PathBuf::from(pd).join("qmpo").join("config.toml"))
    }
    #[cfg(target_os = "macos")]
    {
        Some(PathBuf::from("/Library/Application Support/qmpo/config.toml"))
    }
    #[cfg(target_os = "linux")]
    {
        Some(PathBuf::from("/etc/qmpo/config.toml"))
    }
    #[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
    {
        None
    }
}

/// Returns the user-specific config path.
fn user_config_path() -> Option<PathBuf> {
    let base_dirs = BaseDirs::new()?;

    #[cfg(target_os = "linux")]
    {
        Some(base_dirs.config_dir().join("qmpo").join("config.toml"))
    }
    #[cfg(not(target_os = "linux"))]
    {
        Some(
            base_dirs
                .data_local_dir()
                .join("qmpo")
                .join("config.toml"),
        )
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

    #[test]
    fn test_merge_user_overrides_machine() {
        let machine = Config {
            security: SecurityConfig {
                allowed_servers: vec!["machine-server.com".to_string()],
            },
        };
        let user = PartialConfig {
            security: Some(PartialSecurityConfig {
                allowed_servers: Some(vec!["user-server.com".to_string()]),
            }),
        };
        let merged = machine.merge(user);
        assert_eq!(
            merged.security.allowed_servers,
            vec!["user-server.com"]
        );
    }

    #[test]
    fn test_merge_no_user_security_keeps_machine() {
        let machine = Config {
            security: SecurityConfig {
                allowed_servers: vec!["machine-server.com".to_string()],
            },
        };
        let user = PartialConfig { security: None };
        let merged = machine.merge(user);
        assert_eq!(
            merged.security.allowed_servers,
            vec!["machine-server.com"]
        );
    }

    #[test]
    fn test_merge_user_empty_servers_clears_machine() {
        let machine = Config {
            security: SecurityConfig {
                allowed_servers: vec!["machine-server.com".to_string()],
            },
        };
        let user = PartialConfig {
            security: Some(PartialSecurityConfig {
                allowed_servers: Some(vec![]),
            }),
        };
        let merged = machine.merge(user);
        assert!(merged.security.allowed_servers.is_empty());
    }

    #[test]
    fn test_merge_user_no_allowed_servers_keeps_machine() {
        let machine = Config {
            security: SecurityConfig {
                allowed_servers: vec!["machine-server.com".to_string()],
            },
        };
        let user = PartialConfig {
            security: Some(PartialSecurityConfig {
                allowed_servers: None,
            }),
        };
        let merged = machine.merge(user);
        assert_eq!(
            merged.security.allowed_servers,
            vec!["machine-server.com"]
        );
    }

    #[test]
    fn test_machine_config_path_is_some() {
        assert!(machine_config_path().is_some());
    }

    #[test]
    fn test_user_config_path_is_some() {
        assert!(user_config_path().is_some());
    }
}
