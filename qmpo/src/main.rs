//! qmpo - Open Directory With Browser
//!
//! A directory:// URI scheme handler that opens directories in your file manager.

#![windows_subsystem = "windows"]

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

    // Open in file manager (with file selected if path is a file)
    open_in_file_manager(&canonical_path)?;

    Ok(())
}

/// Open a path in the system's file manager.
/// If the path is a file, opens the parent directory with the file selected.
#[cfg(target_os = "windows")]
fn open_in_file_manager(path: &Path) -> Result<(), Box<dyn std::error::Error>> {
    if path.is_file() {
        // Open parent directory with file selected
        let arg = format!("/select,{}", path.display());
        Command::new("explorer.exe").arg(&arg).spawn()?;
    } else {
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
        let result = Command::new("dbus-send")
            .args([
                "--session",
                "--dest=org.freedesktop.FileManager1",
                "--type=method_call",
                "/org/freedesktop/FileManager1",
                "org.freedesktop.FileManager1.ShowItems",
                &format!("array:string:{}", file_uri),
                "string:",
            ])
            .spawn();

        if result.is_err() {
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
