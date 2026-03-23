use flate2;
use lz4_flex;
use std::{
    io::{self, Read},
    sync::Arc,
};
use zstd;

pub enum CompressionAlgorithm {
    Zlib,
    Lz4,
    Zstd,
    None,
}
impl CompressionAlgorithm {
    pub fn from_compression_level(level: i32) -> Self {
        let algo = level / 100;
        match algo {
            1 => CompressionAlgorithm::Zlib,
            4 => CompressionAlgorithm::Lz4,
            5 => CompressionAlgorithm::Zstd,
            _ => CompressionAlgorithm::None,
        }
    }
    pub fn from_magic(magic: &[u8]) -> Self {
        if magic.len() < 2 {
            return CompressionAlgorithm::None;
        }
        // Matching on bytes is faster and more idiomatic than converting to String
        match (magic[0], magic[1]) {
            (b'Z', b'L') => CompressionAlgorithm::Zlib,
            (b'L', b'4') | (b'C', b'S') => CompressionAlgorithm::Lz4,
            (b'Z', b'S') => CompressionAlgorithm::Zstd,
            _ => CompressionAlgorithm::None,
        }
    }
    pub fn decompress(data: &[u8]) -> io::Result<Arc<[u8]>> {
        let algo = Self::from_magic(data);
        match algo {
            CompressionAlgorithm::Zlib => {
                let mut decoder = flate2::read::ZlibDecoder::new(&data[9..]);
                let _magic = String::from_utf8_lossy(&data[0..2]);
                let compressed_size = u32::from_le_bytes([data[3], data[4], data[5], 0]);
                assert_eq!(compressed_size as usize, data.len() - 9);
                let uncompressed_size = u32::from_le_bytes([data[6], data[7], data[8], 0]);
                let mut decompressed_data = Vec::with_capacity(uncompressed_size as usize);
                decoder.read_to_end(&mut decompressed_data)?;
                Ok(Arc::from(decompressed_data))
            }
            CompressionAlgorithm::Lz4 => {
                let decompressed_data =
                    lz4_flex::decompress_size_prepended(data).map_err(|e| io::Error::other(e))?;
                Ok(Arc::from(decompressed_data))
            }
            CompressionAlgorithm::Zstd => {
                let decompressed_data = zstd::decode_all(data)?;
                Ok(Arc::from(decompressed_data))
            }
            CompressionAlgorithm::None => Ok(Arc::from(data.to_vec())),
        }
    }
}
