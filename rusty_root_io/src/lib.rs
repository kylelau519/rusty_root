pub mod tkey;
pub mod tfile;
pub mod compression;
pub mod tlist;

#[cfg(test)]
mod tests {
    use crate::compression::HasCompressedData;

    use super::*;
    use tfile::{TFileHeader, TFile};
    use tkey::TKeyHeader;
    use compression::decompress;

    #[test]
    fn test_read_streaming_info() {
        let path = "/Users/kylelau519/Programming/rusty_root/rusty_root_io/testfiles/output.root";
        let mut tfile = TFile::open(path).expect("Failed to open ROOT file");
        let tkey_offset = tfile.header.f_seek_info;
        let f_units = tfile.header.f_units;
        let key = TKeyHeader::read_tkey_at(tfile.reader_mut(), tkey_offset, f_units).expect("Failed to read TKey at offset");
        dbg!(&key);
        assert!(key.class_name == "TList");
        assert!(key.name == "StreamerInfo");
    }

    #[test]
    fn test_decompression_on_file() {
        let path = "/Users/kylelau519/Programming/rusty_root/rusty_root_io/testfiles/output.root";
        let mut tfile = TFile::open(path).expect("Failed to open ROOT file");
        assert_eq!(tfile.streamer_info.n_bytes, tfile.streamer_info.compressed_data.len() as u32 + tfile.streamer_info.key_len as u32);
        let compression_level = tfile.header.f_compress;
        let data = decompress(&tfile.streamer_info.compressed_data, compression_level);
        assert!(data.is_ok());
        assert!(data.unwrap().len() == tfile.streamer_info.obj_len as usize);
    }

    use crate::tlist::TList;
    #[test]
    fn test_read_root_file() {
        
        let path = "/Users/kylelau519/Programming/rusty_root/rusty_root_io/testfiles/output.root";
        let mut tfile = TFile::open(path).expect("Failed to open ROOT file");
        tfile.streamer_info.decompress_and_store( tfile.header.f_compress).expect("Failed to decompress streamer info");
        let tlist = TList::new_from_data(tfile.streamer_info.decompressed_data().unwrap());
        // dbg!(&tlist.byte_count);
        assert!(tlist.is_ok());
        

    }
}
