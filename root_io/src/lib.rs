//! ROOT file I/O library for reading and analyzing CERN ROOT files
//! 
//! This library provides functionality to read ROOT files and extract data
//! for analysis and visualization.

pub mod error;
pub mod file;
pub mod tree;
pub mod histogram;
pub mod reader;

pub use error::{RootError, Result};
pub use file::RootFile;
pub use tree::Tree;
pub use histogram::Histogram;
pub use reader::RootReader;

/// Version of the ROOT I/O library
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version_exists() {
        assert!(!VERSION.is_empty());
    }
}