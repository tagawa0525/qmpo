//! qmpo-core - Core library for qmpo (Open Directory With Browser)
//!
//! Provides URI parsing and path conversion for the `directory://` custom URI scheme.
//!
//! # Overview
//!
//! This crate parses `directory://` URIs and converts them to native filesystem paths.
//! It supports Windows local paths, Windows UNC paths, and Unix paths.
//!
//! # URI Format
//!
//! | OS | Filesystem Path | URI |
//! |----|-----------------|-----|
//! | Windows (local) | `C:\Users\tagawa` | `directory://C:/Users/tagawa` |
//! | Windows (UNC) | `\\server\share\folder` | `directory://server/share/folder` |
//! | Unix | `/home/tagawa` | `directory:///home/tagawa` |
//!
//! # Example
//!
//! ```
//! use qmpo_core::DirectoryUri;
//!
//! // Parse a Unix path
//! let uri = DirectoryUri::parse("directory:///home/tagawa").unwrap();
//! assert_eq!(uri.path().to_str().unwrap(), "/home/tagawa");
//!
//! // Parse a Windows path
//! let uri = DirectoryUri::parse("directory://C:/Users/tagawa").unwrap();
//! assert_eq!(uri.path().to_str().unwrap(), "C:\\Users\\tagawa");
//! ```

pub mod error;
pub mod uri;

pub use error::{QmpoError, Result};
pub use uri::DirectoryUri;
