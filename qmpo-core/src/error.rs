//! Error types for qmpo-core.

use thiserror::Error;

/// Errors that can occur during URI parsing and path conversion.
///
/// This enum covers all error conditions that [`DirectoryUri::parse`](crate::DirectoryUri::parse)
/// can encounter.
///
/// # Example
///
/// ```
/// use qmpo_core::{DirectoryUri, QmpoError};
///
/// let result = DirectoryUri::parse("file:///home/user");
/// assert!(matches!(result, Err(QmpoError::InvalidScheme(_))));
/// ```
#[derive(Debug, Error)]
pub enum QmpoError {
    /// The URI has an invalid scheme (not `directory`).
    #[error("invalid URI scheme: expected 'directory', got '{0}'")]
    InvalidScheme(String),

    /// The URI format is invalid or malformed.
    #[error("invalid URI format: {0}")]
    InvalidUri(String),

    /// The URI contains an empty path component.
    #[error("empty path in URI")]
    EmptyPath,

    /// Failed to parse the URI with the `url` crate.
    #[error("URL parse error: {0}")]
    UrlParse(#[from] url::ParseError),

    /// The percent-decoded path contains invalid UTF-8.
    #[error("UTF-8 decode error: {0}")]
    Utf8Error(#[from] std::string::FromUtf8Error),

    /// An I/O error occurred.
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

/// A specialized [`Result`](std::result::Result) type for qmpo operations.
pub type Result<T> = std::result::Result<T, QmpoError>;
