//! Error types for qmpo-lau.

use std::io;
use thiserror::Error;

/// Errors that can occur during qmpo-lau operations.
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum LauError {
    /// Failed to determine user directories.
    #[error("could not determine user directories")]
    NoUserDirectories,

    /// The qmpo executable was not found.
    #[error("qmpo executable not found at: {0}")]
    ExecutableNotFound(String),

    /// The qmpo executable could not be located automatically.
    #[error("could not find qmpo executable; please specify --path")]
    ExecutableNotLocated,

    /// Path contains invalid characters.
    #[error("path contains invalid characters: {0}")]
    InvalidPath(String),

    /// IO error during file operations.
    #[error("IO error: {0}")]
    Io(#[from] io::Error),

    /// Failed to execute external command.
    #[error("command failed: {0}")]
    CommandFailed(String),

    /// Registry operation failed (Windows).
    #[error("registry error: {0}")]
    Registry(String),

    /// Launch Services operation failed (macOS).
    #[error("launch services error: {0}")]
    LaunchServices(String),

    /// XDG MIME operation failed (Linux).
    #[error("XDG MIME error: {0}")]
    XdgMime(String),

    /// Plist operation failed (macOS).
    #[cfg(target_os = "macos")]
    #[error("plist error: {0}")]
    Plist(#[from] plist::Error),
}

pub type Result<T> = std::result::Result<T, LauError>;
