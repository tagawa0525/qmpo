//! qmpo - Open Directory With Browser
//!
//! A directory:// URI scheme handler that opens directories in your file manager.

#![windows_subsystem = "windows"]

mod config;
mod error;
mod log;
mod uri;

use std::net::{IpAddr, ToSocketAddrs};
#[cfg(target_os = "windows")]
use std::os::windows::process::CommandExt;
use std::path::Path;
use std::process::Command;

use clap::Parser;
use uri::DirectoryUri;

#[derive(Parser, Debug)]
#[command(name = "qmpo")]
#[command(about = "Open Directory With Browser - directory:// URI handler")]
#[command(version)]
struct Args {
    /// The directory URI to open (e.g., directory:///home/user)
    uri: String,
}

fn main() {
    let args = Args::parse();

    log::info(&format!("Received URI: {}", args.uri));

    if let Err(e) = run(&args.uri) {
        log::error(&format!("Failed: {}", e));
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }

    log::info("Completed successfully");
}

fn run(uri_str: &str) -> Result<(), Box<dyn std::error::Error>> {
    let uri = DirectoryUri::parse(uri_str)?;
    let path = uri.path();

    log::info(&format!("Parsed path: {}", path.display()));

    // Block UNC paths targeting non-private servers to prevent NTLM hash leaks.
    // path.exists() and canonicalize() trigger SMB connections that send NTLM
    // credentials automatically, so this check must happen before any filesystem access.
    #[cfg(target_os = "windows")]
    if let Some(server) = extract_unc_server(path) {
        validate_unc_server(&server)?;
    }

    if !path.exists() {
        return Err(format!("Path does not exist: {}", path.display()).into());
    }

    // Canonicalize to resolve symlinks and normalize the path.
    // This is best-effort normalization only — when the fallback below is taken,
    // no symlink resolution or traversal prevention is guaranteed.
    // For UNC network paths with ABE (Access-Based Enumeration), canonicalize()
    // may fail with Access Denied (os error 5) even when the path is accessible,
    // because it traverses each ancestor directory internally.
    // The URI parser already produces a proper \\server\... path, so fall back
    // to the original path as-is.
    let canonical_path = match path.canonicalize() {
        Ok(p) => p,
        Err(e) => {
            log::info(&format!("canonicalize failed ({}), using original path", e));
            path.to_path_buf()
        }
    };

    log::info(&format!("Opening: {}", canonical_path.display()));

    // Open in file manager (with file selected if path is a file)
    open_in_file_manager(&canonical_path)?;

    Ok(())
}

/// Open a path in the system's file manager.
/// If the path is a file, opens the parent directory with the file selected.
#[cfg(target_os = "windows")]
fn open_in_file_manager(path: &Path) -> Result<(), Box<dyn std::error::Error>> {
    // path.is_file() can return false on UNC paths due to permission checks,
    // even when the file exists and is accessible. As a fallback, when both
    // is_file() and is_dir() report false but the path exists and has an
    // extension, treat it as a file-like target.
    let treat_as_file = if path.is_file() {
        true
    } else if path.is_dir() {
        false
    } else {
        path.exists() && path.extension().is_some()
    };

    if treat_as_file {
        // explorer.exe uses its own command-line parser for /select,<path>.
        // Paths with special characters like ( ) 【 】 ～ break this parser
        // unless the path is wrapped in double-quotes.
        // raw_arg() bypasses Rust's automatic quoting so we control the exact string.
        let raw = format!("/select,\"{}\"", path.to_string_lossy());
        Command::new("explorer.exe").raw_arg(&raw).spawn()?;
    } else {
        // For the plain directory case, rely on Rust/Windows CreateProcess
        // argument handling instead of manual quoting, so that paths ending
        // with a backslash (e.g. drive roots like D:\) are handled correctly.
        Command::new("explorer.exe").arg(path).spawn()?;
    }
    Ok(())
}

/// Open a path in the system's file manager.
/// If the path is a file, opens the parent directory with the file selected.
#[cfg(target_os = "macos")]
fn open_in_file_manager(path: &Path) -> Result<(), Box<dyn std::error::Error>> {
    if path.is_file() {
        // Open parent directory with file selected using -R flag
        Command::new("open").arg("-R").arg(path).spawn()?;
    } else {
        Command::new("open").arg(path).spawn()?;
    }
    Ok(())
}

/// Open a path in the system's file manager.
/// If the path is a file, attempts to open the parent directory with the file selected.
#[cfg(target_os = "linux")]
fn open_in_file_manager(path: &Path) -> Result<(), Box<dyn std::error::Error>> {
    if path.is_file() {
        // Try dbus-send to select file in file manager (works with Nautilus, Dolphin, etc.)
        let file_uri = format!("file://{}", path.display());
        let dbus_result = Command::new("dbus-send")
            .args([
                "--session",
                "--dest=org.freedesktop.FileManager1",
                "--type=method_call",
                "/org/freedesktop/FileManager1",
                "org.freedesktop.FileManager1.ShowItems",
                &format!("array:string:{}", file_uri),
                "string:",
            ])
            .status();

        let dbus_succeeded = dbus_result.map(|s| s.success()).unwrap_or(false);

        if !dbus_succeeded {
            // Fallback: open parent directory without file selection
            if let Some(parent) = path.parent() {
                Command::new("xdg-open").arg(parent).spawn()?;
            }
        }
    } else {
        Command::new("xdg-open").arg(path).spawn()?;
    }
    Ok(())
}

/// Open a directory in the system's file manager.
#[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
fn open_in_file_manager(_path: &Path) -> Result<(), Box<dyn std::error::Error>> {
    Err("Unsupported operating system".into())
}

/// Extract the server name from a UNC path (e.g., `\\server\share` → `server`).
fn extract_unc_server(path: &Path) -> Option<String> {
    let s = path.to_str()?;
    let rest = s.strip_prefix("\\\\")?;
    let server = rest.split('\\').next().filter(|s| !s.is_empty())?;
    Some(server.to_string())
}

/// Verify that a UNC server is either whitelisted in config.toml or resolves
/// only to private/link-local IP addresses.
/// Rejects external servers to prevent NTLM credential leaks via rogue SMB servers.
///
/// Note: There is an inherent TOCTOU gap between this DNS check and the subsequent
/// filesystem access (`path.exists()`). Exploiting this would require DNS cache
/// poisoning between the two calls, which is a low-probability attack vector.
fn validate_unc_server(server: &str) -> Result<(), Box<dyn std::error::Error>> {
    validate_unc_server_with_config(server, &config::Config::load())
}

fn validate_unc_server_with_config(
    server: &str,
    config: &config::Config,
) -> Result<(), Box<dyn std::error::Error>> {
    if config.security.is_server_allowed(server) {
        return Ok(());
    }

    if let Ok(ip) = server.parse::<IpAddr>() {
        if !is_private_ip(ip) {
            return Err(format!(
                "UNC target {server} is a non-private IP address; blocked to prevent NTLM leak"
            )
            .into());
        }
        return Ok(());
    }

    let addrs: Vec<_> = match (server, 445u16).to_socket_addrs() {
        Ok(iter) => iter.collect(),
        Err(_) => {
            // DNS resolution failed — no SMB connection will happen, safe to proceed.
            // path.exists() will return false and the caller will report the error.
            return Ok(());
        }
    };

    for addr in &addrs {
        if !is_private_ip(addr.ip()) {
            return Err(format!(
                "UNC target {server} resolves to non-private IP {}; blocked to prevent NTLM leak",
                addr.ip()
            )
            .into());
        }
    }

    Ok(())
}

fn is_private_ip(ip: IpAddr) -> bool {
    match ip {
        IpAddr::V4(v4) => v4.is_private() || v4.is_loopback() || v4.is_link_local(),
        IpAddr::V6(v6) => {
            v6.is_loopback()
                || (v6.segments()[0] & 0xfe00) == 0xfc00  // fc00::/7 unique local
                || (v6.segments()[0] & 0xffc0) == 0xfe80  // fe80::/10 link-local
        }
    }
}

#[cfg(test)]
mod unc_tests {
    use super::*;
    use std::path::PathBuf;

    // extract_unc_server tests
    #[test]
    fn test_extract_unc_server_basic() {
        let path = PathBuf::from(r"\\fileserver\share\folder");
        assert_eq!(extract_unc_server(&path).as_deref(), Some("fileserver"));
    }

    #[test]
    fn test_extract_unc_server_ip() {
        let path = PathBuf::from(r"\\192.168.1.100\share");
        assert_eq!(extract_unc_server(&path).as_deref(), Some("192.168.1.100"));
    }

    #[test]
    fn test_extract_unc_server_not_unc() {
        let path = PathBuf::from(r"C:\Users\tagawa");
        assert_eq!(extract_unc_server(&path), None);
    }

    #[test]
    fn test_extract_unc_server_unix_path() {
        let path = PathBuf::from("/home/user");
        assert_eq!(extract_unc_server(&path), None);
    }

    #[test]
    fn test_extract_unc_server_bare_prefix() {
        let path = PathBuf::from(r"\\");
        assert_eq!(extract_unc_server(&path), None);
    }

    // is_private_ip tests
    #[test]
    fn test_private_ipv4() {
        assert!(is_private_ip("10.0.0.1".parse().unwrap()));
        assert!(is_private_ip("172.16.0.1".parse().unwrap()));
        assert!(is_private_ip("172.31.255.255".parse().unwrap()));
        assert!(is_private_ip("192.168.0.1".parse().unwrap()));
    }

    #[test]
    fn test_loopback() {
        assert!(is_private_ip("127.0.0.1".parse().unwrap()));
        assert!(is_private_ip("::1".parse().unwrap()));
    }

    #[test]
    fn test_link_local_ipv4() {
        assert!(is_private_ip("169.254.1.1".parse().unwrap()));
    }

    #[test]
    fn test_public_ipv4() {
        assert!(!is_private_ip("8.8.8.8".parse().unwrap()));
        assert!(!is_private_ip("203.0.113.1".parse().unwrap()));
        assert!(!is_private_ip("1.1.1.1".parse().unwrap()));
        // 172.32.0.1 is just outside the 172.16-31 private range
        assert!(!is_private_ip("172.32.0.1".parse().unwrap()));
    }

    #[test]
    fn test_ipv6_unique_local() {
        assert!(is_private_ip("fd12:3456:789a::1".parse().unwrap()));
    }

    #[test]
    fn test_ipv6_link_local() {
        assert!(is_private_ip("fe80::1".parse().unwrap()));
    }

    #[test]
    fn test_ipv6_public() {
        assert!(!is_private_ip("2001:db8::1".parse().unwrap()));
    }

    fn empty_config() -> config::Config {
        config::Config::default()
    }

    fn config_with_allowed(servers: &[&str]) -> config::Config {
        config::Config {
            security: config::SecurityConfig {
                allowed_servers: servers.iter().map(|s| s.to_string()).collect(),
            },
        }
    }

    // validate_unc_server tests
    #[test]
    fn test_validate_private_ip_server() {
        let cfg = empty_config();
        assert!(validate_unc_server_with_config("192.168.1.1", &cfg).is_ok());
        assert!(validate_unc_server_with_config("10.0.0.1", &cfg).is_ok());
        assert!(validate_unc_server_with_config("172.16.0.1", &cfg).is_ok());
    }

    #[test]
    fn test_validate_public_ip_server() {
        let cfg = empty_config();
        assert!(validate_unc_server_with_config("8.8.8.8", &cfg).is_err());
        assert!(validate_unc_server_with_config("203.0.113.1", &cfg).is_err());
    }

    #[test]
    fn test_validate_loopback_server() {
        let cfg = empty_config();
        assert!(validate_unc_server_with_config("127.0.0.1", &cfg).is_ok());
    }

    #[test]
    fn test_validate_unresolvable_server() {
        let cfg = empty_config();
        assert!(validate_unc_server_with_config("this-host-does-not-exist-qmpo.invalid", &cfg).is_ok());
    }

    #[test]
    fn test_validate_whitelisted_public_ip() {
        let cfg = config_with_allowed(&["203.0.113.50"]);
        assert!(validate_unc_server_with_config("203.0.113.50", &cfg).is_ok());
        // Non-whitelisted public IP still blocked
        assert!(validate_unc_server_with_config("203.0.113.51", &cfg).is_err());
    }

    #[test]
    fn test_validate_whitelisted_hostname() {
        let cfg = config_with_allowed(&["fileserver.example.com"]);
        assert!(validate_unc_server_with_config("fileserver.example.com", &cfg).is_ok());
        assert!(validate_unc_server_with_config("FILESERVER.EXAMPLE.COM", &cfg).is_ok());
    }
}
