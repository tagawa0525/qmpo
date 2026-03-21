//! qmpo-lau - Registration tool for qmpo (Open Directory With Browser)
//!
//! Registers qmpo as the directory:// URI scheme handler on your system.

use std::path::PathBuf;

use clap::{Parser, Subcommand};

mod error;
mod linux;
mod macos;
mod windows;

pub use error::{LauError, Result};

/// Check write access to an install directory and return a helpful error if insufficient.
pub fn check_install_permissions(dir: &std::path::Path, hint: &str) -> Result<()> {
    match std::fs::create_dir_all(dir) {
        Ok(()) => Ok(()),
        Err(e) if e.kind() == std::io::ErrorKind::PermissionDenied => {
            Err(LauError::PermissionDenied {
                operation: format!("installing to {}", dir.display()),
                hint: hint.into(),
            })
        }
        Err(e) => Err(LauError::Io(e)),
    }
}

#[derive(Parser, Debug)]
#[command(name = "qmpo-lau")]
#[command(about = "Registration tool for qmpo - Open Directory With Browser")]
#[command(version)]
struct Args {
    #[command(subcommand)]
    command: Option<Command>,

    /// Path to qmpo executable (auto-detected if not specified)
    /// Used when running without subcommand (defaults to register)
    #[arg(long, global = true)]
    path: Option<PathBuf>,
}

#[derive(Subcommand, Debug)]
enum Command {
    /// Register qmpo as the directory:// URI handler (default)
    Register {
        /// Path to qmpo executable (auto-detected if not specified)
        #[arg(long)]
        path: Option<PathBuf>,
    },
    /// Unregister qmpo as the directory:// URI handler
    Unregister,
    /// Show registration status
    Status,
}

fn main() {
    let args = Args::parse();

    let result = match args.command {
        Some(Command::Register { path }) => register(path),
        Some(Command::Unregister) => unregister(),
        Some(Command::Status) => status(),
        // Default to register when no subcommand is provided (e.g., double-click)
        None => register(args.path),
    };

    if let Err(e) = result {
        eprintln!("Error: {e}");
        std::process::exit(1);
    }
}

#[cfg(target_os = "windows")]
const QMPO_EXECUTABLE_NAME: &str = "qmpo.exe";
#[cfg(not(target_os = "windows"))]
const QMPO_EXECUTABLE_NAME: &str = "qmpo";

/// Find qmpo executable in common locations.
pub fn find_qmpo_executable() -> Result<PathBuf> {
    let current_exe_dir = std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|p| p.to_path_buf()));
    let current_dir = std::env::current_dir().map_err(LauError::Io)?;

    let mut candidates = Vec::new();

    // Same directory as qmpo-lau
    if let Some(dir) = current_exe_dir {
        candidates.push(dir.join(QMPO_EXECUTABLE_NAME));
    }

    // Current directory
    candidates.push(current_dir.join(QMPO_EXECUTABLE_NAME));

    // Machine-wide install location
    #[cfg(target_os = "windows")]
    if let Ok(pf) = std::env::var("PROGRAMFILES") {
        candidates.push(PathBuf::from(pf).join("qmpo").join(QMPO_EXECUTABLE_NAME));
    }
    #[cfg(target_os = "macos")]
    candidates.push(PathBuf::from(macos::APPLICATIONS_DIR).join("qmpo.app/Contents/MacOS/qmpo"));
    #[cfg(target_os = "linux")]
    candidates.push(PathBuf::from(linux::INSTALL_DIR).join(QMPO_EXECUTABLE_NAME));

    // target/release and target/debug
    candidates.push(
        current_dir
            .join("target/release")
            .join(QMPO_EXECUTABLE_NAME),
    );
    candidates.push(current_dir.join("target/debug").join(QMPO_EXECUTABLE_NAME));

    for candidate in candidates {
        if candidate.exists() {
            return Ok(candidate);
        }
    }

    Err(LauError::ExecutableNotLocated)
}

#[cfg(target_os = "windows")]
fn register(path: Option<PathBuf>) -> Result<()> {
    windows::register(path)
}

#[cfg(target_os = "macos")]
fn register(path: Option<PathBuf>) -> Result<()> {
    macos::register(path)
}

#[cfg(target_os = "linux")]
fn register(path: Option<PathBuf>) -> Result<()> {
    linux::register(path)
}

#[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
fn register(_path: Option<PathBuf>) -> Result<()> {
    Err(LauError::CommandFailed(
        "Unsupported operating system".into(),
    ))
}

#[cfg(target_os = "windows")]
fn unregister() -> Result<()> {
    windows::unregister()
}

#[cfg(target_os = "macos")]
fn unregister() -> Result<()> {
    macos::unregister()
}

#[cfg(target_os = "linux")]
fn unregister() -> Result<()> {
    linux::unregister()
}

#[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
fn unregister() -> Result<()> {
    Err(LauError::CommandFailed(
        "Unsupported operating system".into(),
    ))
}

#[cfg(target_os = "windows")]
fn status() -> Result<()> {
    windows::status()
}

#[cfg(target_os = "macos")]
fn status() -> Result<()> {
    macos::status()
}

#[cfg(target_os = "linux")]
fn status() -> Result<()> {
    linux::status()
}

#[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
fn status() -> Result<()> {
    Err(LauError::CommandFailed(
        "Unsupported operating system".into(),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_find_qmpo_executable_returns_existing_path() {
        // When qmpo exists (e.g. in target/debug or target/release alongside
        // the test binary), find_qmpo_executable should return an existing path.
        // When it doesn't exist, it should return an error.
        match find_qmpo_executable() {
            Ok(path) => assert!(
                path.exists(),
                "returned path should exist: {}",
                path.display()
            ),
            Err(e) => assert!(
                matches!(e, LauError::ExecutableNotLocated),
                "unexpected error: {e}"
            ),
        }
    }

    #[test]
    fn test_qmpo_executable_name() {
        #[cfg(target_os = "windows")]
        assert_eq!(QMPO_EXECUTABLE_NAME, "qmpo.exe");

        #[cfg(not(target_os = "windows"))]
        assert_eq!(QMPO_EXECUTABLE_NAME, "qmpo");
    }
}
