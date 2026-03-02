use crate::compression::decompress;
use crate::constant::{K_BYTECOUNTMASK, K_NEWCLASSTAG};
use crate::tkey::TKey;
use crate::tlist::TList;
use crate::tstreamerinfo::TStreamerInfo;
use std::fs::File;
use std::io;
use std::io::{BufReader, Seek, SeekFrom};

// https://root.cern/doc/v638/streamerinfo.html
#[derive(Default)]
pub struct StreamerInfo {
    pub streamer_info_header: TKey,
    pub tlist: TList<TStreamerInfo>,
}

impl StreamerInfo {
    // pub fn read_streamer_info_at<R: std::io::Read + std::io::Seek>(reader: &mut R, offset: u64) -> io::Result<Self> {
    //     reader.seek(SeekFrom::Start(offset))?;
    //     let streamer_info_header = TKey::read_tkey(reader)?;
    //     let payload_len =
    //         (streamer_info_header.n_bytes - streamer_info_header.key_len as u32) as usize;
    //     let mut payload = vec![0u8; payload_len];
    //     let mut data = decompress(&payload, 100);
    //     assert_eq!(data.len(), streamer_info_header.obj_len as usize);

    // }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tkey::TKey;
    use crate::utils::debug_in_ascii;
    use byteorder::ReadBytesExt;
    use std::fs::File;
    #[test]
    fn test_streamer_info() {
        let path =
            "/Users/kylelau519/Programming/rusty_root/rusty_root_io/testfiles/wzqcd_mc20a.root";
        let streamer_info_offset = 80357582;
        let file = File::open(path).expect("Failed to open ROOT file");
        let mut reader = BufReader::new(file);
        let key = TKey::read_tkey_at(&mut reader, streamer_info_offset)
            .expect("Failed to read TKey at offset");
        assert_eq!(key.title, "Doubly linked list");
        dbg!(&key);
    }

    use crate::compression::decompress;
    use std::io::{Read, Seek, SeekFrom};
    #[test]
    fn test_decode_streamer_info() {
        let path =
            "/Users/kylelau519/Programming/rusty_root/rusty_root_io/testfiles/wzqcd_mc20a.root";
        let streamer_info_offset = 80357582;
        let file = File::open(path).expect("Failed to open ROOT file");
        let mut reader = BufReader::new(file);

        let obj_len: usize = 30257;
        let n_bytes: usize = 8360;
        let key_len: usize = 64;
        let mut data = vec![0u8; n_bytes - key_len as usize];
        reader
            .seek(SeekFrom::Start(streamer_info_offset + key_len as u64))
            .expect("Failed to seek to compressed data");
        reader
            .read_exact(&mut data)
            .expect("Failed to read compressed data");

        // Decompress the data
        let decompressed_data = decompress(&data, 101).expect("Failed to decompress data");
        assert_eq!(decompressed_data.len(), obj_len as usize);
        dbg!(debug_in_ascii(&decompressed_data));
        // Implement Read and Seek for the decompressed data using Cursor
        use std::io::Cursor;
        // let mut decompressed_reader = BufReader::new(Cursor::new(decompressed_data));

        let mut reader = Cursor::new(decompressed_data.clone());

        // Now you can use your existing helper functions
        dbg!(obj_len);
        let byte_count = reader
            .read_u32::<byteorder::BigEndian>()
            .expect("Failed to read byte count")
            & K_BYTECOUNTMASK;
        let version = reader
            .read_u16::<byteorder::BigEndian>()
            .expect("Failed to read version");
        let tobject = crate::tobject::TObject::read_tobject(&mut reader)
            .expect("Failed to read TObject from decompressed data");

        // Check that tobject is 12 bytes
        use std::mem::size_of_val;
        assert_eq!(size_of_val(&tobject), 12, "TObject should be 12 bytes");

        dbg!(byte_count, version, tobject);
        let l_name_byte = reader
            .read_u8()
            .expect("Failed to read l_name_byte from decompressed data");
        dbg!(l_name_byte);
        let n_objects = reader
            .read_u32::<byteorder::BigEndian>()
            .expect("Failed to read n_objects from decompressed data");
        dbg!(n_objects);
    }
}
