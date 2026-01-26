//! qmpo-core - Core library for qmpo (Open Directory With Browser)
//!
//! Provides URI parsing and path conversion for the directory:// scheme.

pub mod error;
pub mod uri;

pub use error::{QmpoError, Result};
pub use uri::DirectoryUri;
