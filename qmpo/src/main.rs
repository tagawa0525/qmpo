//! qmpo - Open Directory With Browser
//!
//! A directory:// URI scheme handler that opens directories in your file manager.

#![windows_subsystem = "windows"]

mod error;
mod log;
mod uri;

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

    if !path.exists() {
        return Err(format!("Path does not exist: {}", path.display()).into());
    }

    // Canonicalize to resolve symlinks and prevent path traversal attacks.
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
        Command::new("explorer.exe")
            .raw_arg(format!("\"{}\"", path.to_string_lossy()))
            .spawn()?;
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
