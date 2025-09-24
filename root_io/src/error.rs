//! Error types for ROOT file operations

use thiserror::Error;

/// Result type alias for ROOT operations
pub type Result<T> = std::result::Result<T, RootError>;

/// Errors that can occur when working with ROOT files
#[derive(Error, Debug)]
pub enum RootError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("Invalid ROOT file format: {0}")]
    InvalidFormat(String),
    
    #[error("Unsupported ROOT version: {0}")]
    UnsupportedVersion(u32),
    
    #[error("Key not found: {0}")]
    KeyNotFound(String),
    
    #[error("Parse error: {0}")]
    ParseError(String),
    
    #[error("Compression error: {0}")]
    CompressionError(String),
    
    #[error("Data type error: {0}")]
    DataTypeError(String),
}