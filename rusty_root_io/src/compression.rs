use flate2;
use lz4_flex;
use zstd;
use std::io::{self, Read};
use std::io::Write;


pub enum CompressionAlgorithm {
    Zlib,
    Lz4,
    Zstd,
    None,
}
impl CompressionAlgorithm {
    pub fn from_compression_level(level: u32) -> Self {
        let algo = level / 100;
        match algo {
            1 => CompressionAlgorithm::Zlib,
            4 => CompressionAlgorithm::Lz4,
            5 => CompressionAlgorithm::Zstd,
            _ => CompressionAlgorithm::None,
        }
    }
}

pub fn decompress (data: &[u8] , compression_level: u32, uncompressed_size: Option<u32>) -> io::Result<Vec<u8>> {
    let algo = CompressionAlgorithm::from_compression_level(compression_level);
    match algo {
        CompressionAlgorithm::Zlib => {
            let mut decoder = flate2::read::ZlibDecoder::new(data);
            let mut decompressed_data = Vec::with_capacity(uncompressed_size.unwrap_or(0) as usize);
            decoder.read_to_end(&mut decompressed_data)?;
            Ok(decompressed_data)
        },
        CompressionAlgorithm::Lz4 => {
            let decompressed_data = lz4_flex::decompress_size_prepended(data)
                .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
            Ok(decompressed_data)
        },
        CompressionAlgorithm::Zstd => {
            let decompressed_data = zstd::decode_all(data)?;
            Ok(decompressed_data)
        },
        CompressionAlgorithm::None => {
            Ok(data.to_vec())
        },
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_decompress() {
        // Generate some sample data and compress it using zlib for testing
        let original_data = b"Hello, world! This is some test data for compression.";
        let mut encoder = flate2::write::ZlibEncoder::new(Vec::new(), flate2::Compression::default());
        encoder.write_all(original_data).unwrap();
        let compressed_data = encoder.finish().unwrap();
        let compression_level = 101; // example level
        let uncompressed_size = Some(1024); // example size
        let result = decompress(&compressed_data, compression_level, uncompressed_size);
        dbg!(String::from_utf8_lossy(&result.as_ref().unwrap()));
        assert!(result.is_ok());
    }
}