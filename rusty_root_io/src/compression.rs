use flate2;
use lz4_flex;
use zstd;
use std::io::{self, Read};


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
}

pub fn decompress (data: &[u8] , compression_level: i32) -> io::Result<Vec<u8>> {
    let algo = CompressionAlgorithm::from_compression_level(compression_level);
    match algo {
        CompressionAlgorithm::Zlib => {
            let mut decoder = flate2::read::ZlibDecoder::new(&data[9..]);
            let _magic = String::from_utf8_lossy(&data[0..2]);
            let _compressed_size = u32::from_le_bytes([data[3], data[4], data[5], 0]);
            let uncompressed_size = u32::from_le_bytes([data[6], data[7], data[8], 0]);
            let mut decompressed_data = Vec::with_capacity(uncompressed_size as usize);
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

pub trait HasCompressedData {
    fn get_compressed_data(&self) -> &[u8];
    fn get_compressed_len(&self) -> usize;
    fn get_uncompressed_len(&self) -> usize;
    fn get_decompressed_data(&self) -> Option<&Vec<u8>>;

    fn decompress_into(&self, compression_level: i32) -> io::Result<Vec<u8>> {
        let compressed_data = self.get_compressed_data();
        decompress(compressed_data, compression_level)
    }

    fn decompress_and_store(&mut self, compression_level: i32) -> io::Result<Vec<u8>> {
        let decompressed_data = self.decompress_into(compression_level)?;
        self.get_decompressed_data().replace(&decompressed_data);
        Ok(decompressed_data)
    }
}


