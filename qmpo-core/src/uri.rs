use std::path::{Path, PathBuf};

use percent_encoding::percent_decode_str;
use url::Url;

use crate::error::{QmpoError, Result};

const SCHEME: &str = "directory";
const SCHEME_PREFIX: &str = "directory://";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DirectoryUri {
    path: PathBuf,
}

impl DirectoryUri {
    /// Parse a directory URI string and convert to a filesystem path.
    ///
    /// # URI Format
    /// - Windows local: `directory://C:/Users/tagawa` -> `C:\Users\tagawa`
    /// - Windows UNC: `directory://server/share/folder` -> `\\server\share\folder`
    /// - Unix: `directory:///home/tagawa` -> `/home/tagawa`
    ///
    /// # Detection Logic
    /// - `directory://X:/...` (single letter followed by colon) -> Windows local path
    /// - `directory://server/...` (no drive letter pattern) -> UNC path
    /// - `directory:///...` (triple slash) -> Unix absolute path
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

    /// Get the filesystem path.
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Consume and return the filesystem path.
    pub fn into_path(self) -> PathBuf {
        self.path
    }
}

fn decode_percent_encoding(s: &str) -> Result<String> {
    percent_decode_str(s)
        .decode_utf8()
        .map(|cow| cow.into_owned())
        .map_err(|e| QmpoError::InvalidUri(format!("invalid UTF-8: {e}")))
}

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

    #[test]
    fn test_unix_path() {
        let uri = DirectoryUri::parse("directory:///home/tagawa").unwrap();
        assert_eq!(uri.path(), &PathBuf::from("/home/tagawa"));
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
    fn test_unc_path() {
        let uri = DirectoryUri::parse("directory://server/share/folder").unwrap();
        assert_eq!(uri.path(), &PathBuf::from("\\\\server\\share\\folder"));
    }

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
}
