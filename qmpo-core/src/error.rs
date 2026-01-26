use thiserror::Error;

#[derive(Debug, Error)]
pub enum QmpoError {
    #[error("invalid URI scheme: expected 'directory', got '{0}'")]
    InvalidScheme(String),

    #[error("invalid URI format: {0}")]
    InvalidUri(String),

    #[error("empty path in URI")]
    EmptyPath,

    #[error("URL parse error: {0}")]
    UrlParse(#[from] url::ParseError),

    #[error("UTF-8 decode error: {0}")]
    Utf8Error(#[from] std::string::FromUtf8Error),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

pub type Result<T> = std::result::Result<T, QmpoError>;
