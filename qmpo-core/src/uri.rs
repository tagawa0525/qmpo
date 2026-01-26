//! URI parsing module for the `directory://` scheme.
//!
//! This module provides [`DirectoryUri`] for parsing and converting URIs to filesystem paths.

use std::path::{Path, PathBuf};

use percent_encoding::percent_decode_str;
use url::Url;

use crate::error::{QmpoError, Result};

/// The URI scheme identifier.
const SCHEME: &str = "directory";
/// The full scheme prefix including the separator.
const SCHEME_PREFIX: &str = "directory://";

/// A parsed `directory://` URI that holds a filesystem path.
///
/// This struct is the main entry point for URI parsing. It handles:
/// - Unix absolute paths: `directory:///path/to/dir`
/// - Windows local paths: `directory://C:/path/to/dir`
/// - Windows UNC paths: `directory://server/share/path`
///
/// # Example
///
/// ```
/// use qmpo_core::DirectoryUri;
///
/// let uri = DirectoryUri::parse("directory:///home/user/documents")?;
/// println!("Path: {}", uri.path().display());
/// # Ok::<(), qmpo_core::QmpoError>(())
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DirectoryUri {
    path: PathBuf,
}

impl DirectoryUri {
    /// Parse a directory URI string and convert to a filesystem path.
    ///
    /// # Arguments
    ///
    /// * `uri_str` - A URI string with the `directory://` scheme
    ///
    /// # URI Format
    ///
    /// | Input | Output |
    /// |-------|--------|
    /// | `directory://C:/Users/tagawa` | `C:\Users\tagawa` |
    /// | `directory://server/share/folder` | `\\server\share\folder` |
    /// | `directory:///home/tagawa` | `/home/tagawa` |
    ///
    /// # Detection Logic
    ///
    /// - `directory://X:/...` (single letter followed by colon) → Windows local path
    /// - `directory://server/...` (no drive letter pattern) → UNC path
    /// - `directory:///...` (triple slash) → Unix absolute path
    ///
    /// # Errors
    ///
    /// Returns [`QmpoError`] if:
    /// - The URI scheme is not `directory`
    /// - The URI format is invalid
    /// - The path is empty
    /// - Percent-encoding contains invalid UTF-8
    ///
    /// # Example
    ///
    /// ```
    /// use qmpo_core::DirectoryUri;
    ///
    /// // Unix path
    /// let uri = DirectoryUri::parse("directory:///home/user")?;
    /// assert_eq!(uri.path().to_str().unwrap(), "/home/user");
    ///
    /// // Percent-encoded path
    /// let uri = DirectoryUri::parse("directory:///home/user/My%20Documents")?;
    /// assert_eq!(uri.path().to_str().unwrap(), "/home/user/My Documents");
    /// # Ok::<(), qmpo_core::QmpoError>(())
    /// ```
    pub fn parse(uri_str: &str) -> Result<Self> {
        let url = Url::parse(uri_str)?;

        if url.scheme() != SCHEME {
            return Err(QmpoError::InvalidScheme(url.scheme().to_string()));
        }

        let path = Self::extract_path(uri_str)?;

        if path.as_os_str().is_empty() {
            return Err(QmpoError::EmptyPath);
        }

        Ok(Self { path })
    }

    fn extract_path(original_uri: &str) -> Result<PathBuf> {
        let after_scheme = original_uri
            .strip_prefix(SCHEME_PREFIX)
            .ok_or_else(|| QmpoError::InvalidUri("missing scheme prefix".to_string()))?;

        if after_scheme.is_empty() {
            return Err(QmpoError::EmptyPath);
        }

        let decoded = decode_percent_encoding(after_scheme)?;

        // Unix absolute path: directory:///home/tagawa -> /home/tagawa
        if after_scheme.starts_with('/') {
            return Ok(PathBuf::from(decoded));
        }

        // Windows drive letter pattern (e.g., C:/)
        if is_windows_drive_letter(&decoded) {
            let windows_path = decoded.replace('/', "\\");
            return Ok(PathBuf::from(windows_path));
        }

        // UNC path: directory://server/share -> \\server\share
        let unc_path = format!("\\\\{}", decoded.replace('/', "\\"));
        Ok(PathBuf::from(unc_path))
    }

    /// Returns a reference to the filesystem path.
    ///
    /// The returned path is already converted to the platform-native format.
    /// Note that the path is not canonicalized - it may contain `..` or symlinks.
    /// Use [`std::path::Path::canonicalize`] if you need an absolute resolved path.
    #[inline]
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Consumes the URI and returns the filesystem path.
    ///
    /// This is useful when you need ownership of the path and no longer need the URI.
    ///
    /// # Example
    ///
    /// ```
    /// use qmpo_core::DirectoryUri;
    /// use std::path::PathBuf;
    ///
    /// let uri = DirectoryUri::parse("directory:///home/user")?;
    /// let path: PathBuf = uri.into_path();
    /// # Ok::<(), qmpo_core::QmpoError>(())
    /// ```
    #[inline]
    pub fn into_path(self) -> PathBuf {
        self.path
    }
}

/// Decode percent-encoded string to UTF-8.
fn decode_percent_encoding(s: &str) -> Result<String> {
    percent_decode_str(s)
        .decode_utf8()
        .map(|cow| cow.into_owned())
        .map_err(|e| QmpoError::InvalidUri(format!("invalid UTF-8: {e}")))
}

/// Check if a string starts with a Windows drive letter pattern (e.g., "C:").
fn is_windows_drive_letter(s: &str) -> bool {
    let mut chars = s.chars();
    matches!(
        (chars.next(), chars.next()),
        (Some(c), Some(':')) if c.is_ascii_alphabetic()
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    // Basic path tests
    #[test]
    fn test_unix_path() {
        let uri = DirectoryUri::parse("directory:///home/tagawa").unwrap();
        assert_eq!(uri.path(), &PathBuf::from("/home/tagawa"));
    }

    #[test]
    fn test_unix_root() {
        let uri = DirectoryUri::parse("directory:///").unwrap();
        assert_eq!(uri.path(), &PathBuf::from("/"));
    }

    #[test]
    fn test_unix_path_with_spaces() {
        let uri = DirectoryUri::parse("directory:///home/tagawa/My%20Documents").unwrap();
        assert_eq!(uri.path(), &PathBuf::from("/home/tagawa/My Documents"));
    }

    #[test]
    fn test_windows_local_path() {
        let uri = DirectoryUri::parse("directory://C:/Users/tagawa").unwrap();
        assert_eq!(uri.path(), &PathBuf::from("C:\\Users\\tagawa"));
    }

    #[test]
    fn test_windows_local_path_lowercase() {
        let uri = DirectoryUri::parse("directory://c:/Users/tagawa").unwrap();
        assert_eq!(uri.path(), &PathBuf::from("c:\\Users\\tagawa"));
    }

    #[test]
    fn test_windows_drive_root() {
        let uri = DirectoryUri::parse("directory://D:/").unwrap();
        assert_eq!(uri.path(), &PathBuf::from("D:\\"));
    }

    #[test]
    fn test_unc_path() {
        let uri = DirectoryUri::parse("directory://server/share/folder").unwrap();
        assert_eq!(uri.path(), &PathBuf::from("\\\\server\\share\\folder"));
    }

    #[test]
    fn test_unc_path_root() {
        let uri = DirectoryUri::parse("directory://server/share").unwrap();
        assert_eq!(uri.path(), &PathBuf::from("\\\\server\\share"));
    }

    // Special characters tests
    #[test]
    fn test_percent_encoded_special_chars() {
        let uri = DirectoryUri::parse("directory:///home/user/%23folder").unwrap();
        assert_eq!(uri.path(), &PathBuf::from("/home/user/#folder"));
    }

    #[test]
    fn test_percent_encoded_unicode() {
        // Japanese characters: テスト
        let uri =
            DirectoryUri::parse("directory:///home/user/%E3%83%86%E3%82%B9%E3%83%88").unwrap();
        assert_eq!(uri.path(), &PathBuf::from("/home/user/テスト"));
    }

    #[test]
    fn test_path_with_dots() {
        // Note: canonicalize() is done at runtime, not during parsing
        let uri = DirectoryUri::parse("directory:///home/user/../other").unwrap();
        assert_eq!(uri.path(), &PathBuf::from("/home/user/../other"));
    }

    // Error tests
    #[test]
    fn test_invalid_scheme() {
        let result = DirectoryUri::parse("file:///home/tagawa");
        assert!(matches!(result, Err(QmpoError::InvalidScheme(_))));
    }

    #[test]
    fn test_empty_path() {
        let result = DirectoryUri::parse("directory://");
        assert!(matches!(result, Err(QmpoError::EmptyPath)));
    }

    #[test]
    fn test_invalid_uri() {
        let result = DirectoryUri::parse("not a uri");
        assert!(result.is_err());
    }

    #[test]
    fn test_missing_scheme() {
        let result = DirectoryUri::parse("///home/tagawa");
        assert!(result.is_err());
    }

    // into_path test
    #[test]
    fn test_into_path() {
        let uri = DirectoryUri::parse("directory:///home/tagawa").unwrap();
        let path: PathBuf = uri.into_path();
        assert_eq!(path, PathBuf::from("/home/tagawa"));
    }
}
