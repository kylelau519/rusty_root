//! ROOT file handling

use std::fs::File;
use std::io::{BufReader, Read, Seek, SeekFrom};
use std::path::Path;
use byteorder::{LittleEndian, ReadBytesExt};
use crate::error::{Result, RootError};

/// ROOT file magic signature
const ROOT_SIGNATURE: &[u8] = b"root";

/// ROOT file header structure
#[derive(Debug, Clone)]
pub struct RootFileHeader {
    pub version: u32,
    pub begin: u32,
    pub end: u32,
    pub seek_free: u32,
    pub n_bytes_free: u32,
    pub n_free: u32,
    pub n_bytes_name: u32,
    pub units: u8,
    pub compression: u32,
    pub seek_info: u32,
    pub n_bytes_info: u32,
}

/// Represents a ROOT file
pub struct RootFile {
    reader: BufReader<File>,
    header: RootFileHeader,
    path: String,
}

impl RootFile {
    /// Open a ROOT file from the given path
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self> {
        let path_str = path.as_ref().to_string_lossy().into_owned();
        let file = File::open(&path)?;
        let mut reader = BufReader::new(file);
        
        let header = Self::read_header(&mut reader)?;
        
        Ok(RootFile {
            reader,
            header,
            path: path_str,
        })
    }
    
    /// Read and validate ROOT file header
    fn read_header(reader: &mut BufReader<File>) -> Result<RootFileHeader> {
        // Read magic signature
        let mut signature = [0u8; 4];
        reader.read_exact(&mut signature)?;
        
        if &signature != ROOT_SIGNATURE {
            return Err(RootError::InvalidFormat(
                "Invalid ROOT file signature".to_string()
            ));
        }
        
        // Read header fields
        let version = reader.read_u32::<LittleEndian>()?;
        let begin = reader.read_u32::<LittleEndian>()?;
        let end = reader.read_u32::<LittleEndian>()?;
        let seek_free = reader.read_u32::<LittleEndian>()?;
        let n_bytes_free = reader.read_u32::<LittleEndian>()?;
        let n_free = reader.read_u32::<LittleEndian>()?;
        let n_bytes_name = reader.read_u32::<LittleEndian>()?;
        let units = reader.read_u8()?;
        let compression = reader.read_u32::<LittleEndian>()?;
        let seek_info = reader.read_u32::<LittleEndian>()?;
        let n_bytes_info = reader.read_u32::<LittleEndian>()?;
        
        Ok(RootFileHeader {
            version,
            begin,
            end,
            seek_free,
            n_bytes_free,
            n_free,
            n_bytes_name,
            units,
            compression,
            seek_info,
            n_bytes_info,
        })
    }
    
    /// Get the file header
    pub fn header(&self) -> &RootFileHeader {
        &self.header
    }
    
    /// Get the file path
    pub fn path(&self) -> &str {
        &self.path
    }
    
    /// Get basic file information as a string
    pub fn info(&self) -> String {
        format!(
            "ROOT File: {}\nVersion: {}\nSize: {} bytes\nCompression: {}",
            self.path,
            self.header.version,
            self.header.end,
            self.header.compression
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use std::fs;

    fn create_mock_root_file() -> Result<String> {
        let temp_path = "/tmp/test_root_file.root";
        let mut file = File::create(temp_path)?;
        
        // Write magic signature
        file.write_all(ROOT_SIGNATURE)?;
        
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
    fn test_open_valid_root_file() {
        let file_path = create_mock_root_file().expect("Failed to create mock file");
        let root_file = RootFile::open(&file_path);
        assert!(root_file.is_ok());
        
        let root_file = root_file.unwrap();
        assert_eq!(root_file.header().version, 1);
        assert_eq!(root_file.path(), file_path);
        
        // Clean up
        let _ = fs::remove_file(file_path);
    }

    #[test]
    fn test_invalid_signature() {
        let temp_path = "/tmp/invalid_root_file.root";
        let mut file = File::create(temp_path).unwrap();
        file.write_all(b"fake").unwrap();
        
        let result = RootFile::open(temp_path);
        assert!(result.is_err());
        
        // Clean up
        let _ = fs::remove_file(temp_path);
    }
}