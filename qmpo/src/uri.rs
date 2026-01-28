//! URI parsing module for the `directory://` scheme.
//!
//! This module provides [`DirectoryUri`] for parsing and converting URIs to filesystem paths.

use std::path::{Path, PathBuf};

use percent_encoding::percent_decode_str;
use url::Url;

use super::error::{QmpoError, Result};

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
            // Also check for Windows drive letter without colon after the leading slash
            // e.g., directory:///C/Windows -> /C/Windows -> C:/Windows
            let path_after_slash = decoded.strip_prefix('/').unwrap_or(&decoded);
            if is_windows_drive_letter_without_colon(path_after_slash) {
                let fixed = fix_windows_drive_letter(path_after_slash);
                let windows_path = fixed.replace('/', "\\");
                return Ok(PathBuf::from(windows_path));
            }
            return Ok(PathBuf::from(decoded));
        }

        // Fix Windows drive letter without colon (e.g., C/Windows -> C:/Windows)
        // Some browsers convert "C:/" to "C/" when handling file:// URLs
        let decoded = fix_windows_drive_letter(&decoded);

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

/// Check if a string starts with a Windows drive letter without colon (e.g., "C/").
/// Some browsers convert "C:/" to "C/" when handling file:// URLs.
fn is_windows_drive_letter_without_colon(s: &str) -> bool {
    let mut chars = s.chars();
    matches!(
        (chars.next(), chars.next()),
        (Some(c), Some('/')) if c.is_ascii_alphabetic()
    )
}

/// Fix Windows drive letter without colon.
/// Converts "C/path" to "C:/path".
fn fix_windows_drive_letter(s: &str) -> String {
    if is_windows_drive_letter_without_colon(s) {
        let mut result = String::with_capacity(s.len() + 1);
        let mut chars = s.chars();
        if let Some(drive) = chars.next() {
            result.push(drive);
            result.push(':');
            result.push_str(chars.as_str());
        }
        result
    } else {
        s.to_string()
    }
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

    // Windows drive letter without colon tests (browser compatibility)
    // Some browsers convert "C:/" to "C/" when handling file:// URLs
    #[test]
    fn test_windows_drive_without_colon() {
        let uri = DirectoryUri::parse("directory://C/Users/tagawa").unwrap();
        assert_eq!(uri.path(), &PathBuf::from("C:\\Users\\tagawa"));
    }

    #[test]
    fn test_windows_drive_without_colon_lowercase() {
        let uri = DirectoryUri::parse("directory://d/Windows").unwrap();
        assert_eq!(uri.path(), &PathBuf::from("d:\\Windows"));
    }

    #[test]
    fn test_windows_drive_without_colon_triple_slash() {
        // directory:///C/Windows -> C:\Windows
        let uri = DirectoryUri::parse("directory:///C/Windows").unwrap();
        assert_eq!(uri.path(), &PathBuf::from("C:\\Windows"));
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
}
