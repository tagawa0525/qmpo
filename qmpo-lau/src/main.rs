//! qmpo-lau - Registration tool for qmpo (Open Directory With Browser)
//!
//! Registers qmpo as the directory:// URI scheme handler on your system.

use std::path::PathBuf;

use clap::{Parser, Subcommand};

mod linux;
mod macos;
mod windows;

#[derive(Parser, Debug)]
#[command(name = "qmpo-lau")]
#[command(about = "Registration tool for qmpo - Open Directory With Browser")]
#[command(version)]
struct Args {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand, Debug)]
enum Command {
    /// Register qmpo as the directory:// URI handler
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

type BoxError = Box<dyn std::error::Error>;

fn main() {
    let args = Args::parse();

    let result = match args.command {
        Command::Register { path } => register(path),
        Command::Unregister => unregister(),
        Command::Status => status(),
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
fn find_qmpo_executable() -> Result<PathBuf, BoxError> {
    let current_exe_dir = std::env::current_exe()?.parent().map(|p| p.to_path_buf());
    let current_dir = std::env::current_dir()?;

    let mut candidates = Vec::new();

    // Same directory as qmpo-lau
    if let Some(dir) = current_exe_dir {
        candidates.push(dir.join(QMPO_EXECUTABLE_NAME));
    }

    // Current directory
    candidates.push(current_dir.join(QMPO_EXECUTABLE_NAME));

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

    Err(format!("Could not find {QMPO_EXECUTABLE_NAME}. Please specify --path").into())
}

#[cfg(target_os = "windows")]
fn register(path: Option<PathBuf>) -> Result<(), BoxError> {
    windows::register(path)
}

#[cfg(target_os = "macos")]
fn register(path: Option<PathBuf>) -> Result<(), BoxError> {
    macos::register(path)
}

#[cfg(target_os = "linux")]
fn register(path: Option<PathBuf>) -> Result<(), BoxError> {
    linux::register(path)
}

#[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
fn register(_path: Option<PathBuf>) -> Result<(), BoxError> {
    Err("Unsupported operating system".into())
}

#[cfg(target_os = "windows")]
fn unregister() -> Result<(), BoxError> {
    windows::unregister()
}

#[cfg(target_os = "macos")]
fn unregister() -> Result<(), BoxError> {
    macos::unregister()
}

#[cfg(target_os = "linux")]
fn unregister() -> Result<(), BoxError> {
    linux::unregister()
}

#[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
fn unregister() -> Result<(), BoxError> {
    Err("Unsupported operating system".into())
}

#[cfg(target_os = "windows")]
fn status() -> Result<(), BoxError> {
    windows::status()
}

#[cfg(target_os = "macos")]
fn status() -> Result<(), BoxError> {
    macos::status()
}

#[cfg(target_os = "linux")]
fn status() -> Result<(), BoxError> {
    linux::status()
}

#[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
fn status() -> Result<(), BoxError> {
    Err("Unsupported operating system".into())
}
