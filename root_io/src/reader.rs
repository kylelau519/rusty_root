//! ROOT file reader with high-level interface

use crate::error::{Result, RootError};
use crate::file::RootFile;
use crate::tree::Tree;
use crate::histogram::Histogram;
use std::path::Path;

/// High-level ROOT file reader
pub struct RootReader {
    file: RootFile,
}

impl RootReader {
    /// Open a ROOT file for reading
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self> {
        let file = RootFile::open(path)?;
        Ok(Self { file })
    }
    
    /// Get file information
    pub fn file_info(&self) -> String {
        self.file.info()
    }
    
    /// List available objects in the file
    pub fn list_objects(&self) -> Vec<String> {
        // For now, return mock objects
        // In a real implementation, this would parse the file directory structure
        vec![
            "sample_tree (TTree)".to_string(),
            "sample_hist (TH1F)".to_string(),
            "energy_spectrum (TH1D)".to_string(),
            "detector_data (TTree)".to_string(),
        ]
    }
    
    /// Read a tree by name (mock implementation)
    pub fn read_tree(&self, name: &str) -> Result<Tree> {
        match name {
            "sample_tree" | "detector_data" => Ok(Tree::create_sample()),
            _ => Err(RootError::KeyNotFound(name.to_string())),
        }
    }
    
    /// Read a histogram by name (mock implementation)
    pub fn read_histogram(&self, name: &str) -> Result<Histogram> {
        match name {
            "sample_hist" | "energy_spectrum" => Ok(Histogram::create_sample()),
            _ => Err(RootError::KeyNotFound(name.to_string())),
        }
    }
    
    /// Get the underlying ROOT file
    pub fn file(&self) -> &RootFile {
        &self.file
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::io::Write;

    fn create_mock_root_file() -> Result<String> {
        let temp_path = "/tmp/test_reader_file.root";
        let mut file = File::create(temp_path)?;
        
        // Write magic signature
        file.write_all(b"root")?;
        
        // Write header fields (simplified)
        file.write_all(&1u32.to_le_bytes())?; // version
        file.write_all(&100u32.to_le_bytes())?; // begin
        file.write_all(&1000u32.to_le_bytes())?; // end
        file.write_all(&0u32.to_le_bytes())?; // seek_free
        file.write_all(&0u32.to_le_bytes())?; // n_bytes_free
        file.write_all(&0u32.to_le_bytes())?; // n_free
        file.write_all(&0u32.to_le_bytes())?; // n_bytes_name
        file.write_all(&[0u8])?; // units
        file.write_all(&0u32.to_le_bytes())?; // compression
        file.write_all(&0u32.to_le_bytes())?; // seek_info
        file.write_all(&0u32.to_le_bytes())?; // n_bytes_info
        
        Ok(temp_path.to_string())
    }

    #[test]
    fn test_open_reader() {
        let file_path = create_mock_root_file().expect("Failed to create mock file");
        let reader = RootReader::open(&file_path);
        assert!(reader.is_ok());
        
        let reader = reader.unwrap();
        let info = reader.file_info();
        assert!(info.contains("ROOT File"));
        
        // Clean up
        let _ = std::fs::remove_file(file_path);
    }

    #[test]
    fn test_list_objects() {
        let file_path = create_mock_root_file().expect("Failed to create mock file");
        let reader = RootReader::open(&file_path).unwrap();
        
        let objects = reader.list_objects();
        assert!(!objects.is_empty());
        assert!(objects.iter().any(|obj| obj.contains("TTree")));
        assert!(objects.iter().any(|obj| obj.contains("TH1")));
        
        // Clean up
        let _ = std::fs::remove_file(file_path);
    }

    #[test]
    fn test_read_tree() {
        let file_path = create_mock_root_file().expect("Failed to create mock file");
        let reader = RootReader::open(&file_path).unwrap();
        
        let tree = reader.read_tree("sample_tree");
        assert!(tree.is_ok());
        
        let tree = tree.unwrap();
        assert_eq!(tree.name, "sample_tree");
        assert!(!tree.branches.is_empty());
        
        // Test non-existent tree
        let result = reader.read_tree("non_existent");
        assert!(result.is_err());
        
        // Clean up
        let _ = std::fs::remove_file(file_path);
    }

    #[test]
    fn test_read_histogram() {
        let file_path = create_mock_root_file().expect("Failed to create mock file");
        let reader = RootReader::open(&file_path).unwrap();
        
        let hist = reader.read_histogram("sample_hist");
        assert!(hist.is_ok());
        
        let hist = hist.unwrap();
        assert_eq!(hist.name, "sample_hist");
        assert!(!hist.bins.is_empty());
        
        // Test non-existent histogram
        let result = reader.read_histogram("non_existent");
        assert!(result.is_err());
        
        // Clean up
        let _ = std::fs::remove_file(file_path);
    }
}