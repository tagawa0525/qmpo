//! qmpo - Open Directory With Browser
//!
//! A directory:// URI scheme handler that opens directories in your file manager.

use std::path::Path;
use std::process::Command;

use clap::Parser;
use qmpo_core::DirectoryUri;

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

    if let Err(e) = run(&args.uri) {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}

fn run(uri_str: &str) -> Result<(), Box<dyn std::error::Error>> {
    let uri = DirectoryUri::parse(uri_str)?;
    let path = uri.path();

    if !path.exists() {
        return Err(format!("Path does not exist: {}", path.display()).into());
    }

    // Canonicalize to resolve symlinks and prevent path traversal attacks
    let canonical_path = path
        .canonicalize()
        .map_err(|e| format!("Failed to resolve path {}: {}", path.display(), e))?;

    // If path is a file, open its parent directory
    let dir = if canonical_path.is_file() {
        canonical_path
            .parent()
            .ok_or_else(|| {
                format!(
                    "Could not get parent directory of: {}",
                    canonical_path.display()
                )
            })?
            .to_path_buf()
    } else {
        canonical_path
    };

    open_in_file_manager(&dir)?;

    Ok(())
}

/// Open a directory in the system's file manager.
#[cfg(target_os = "windows")]
fn open_in_file_manager(path: &Path) -> Result<(), Box<dyn std::error::Error>> {
    Command::new("explorer.exe").arg(path).spawn()?;
    Ok(())
}

/// Open a directory in the system's file manager.
#[cfg(target_os = "macos")]
fn open_in_file_manager(path: &Path) -> Result<(), Box<dyn std::error::Error>> {
    Command::new("open").arg(path).spawn()?;
    Ok(())
}

/// Open a directory in the system's file manager.
#[cfg(target_os = "linux")]
fn open_in_file_manager(path: &Path) -> Result<(), Box<dyn std::error::Error>> {
    Command::new("xdg-open").arg(path).spawn()?;
    Ok(())
}

/// Open a directory in the system's file manager.
#[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
fn open_in_file_manager(_path: &Path) -> Result<(), Box<dyn std::error::Error>> {
    Err("Unsupported operating system".into())
}
