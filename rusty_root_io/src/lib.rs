pub mod tkey;
pub mod tfile;
pub mod compression;
pub mod tlist;
pub mod constant;
pub mod streamerinfo;
pub mod tbuf;

#[cfg(test)]
mod tests {
    use crate::compression::HasCompressedData;

    use super::*;
    use tfile::{TFileHeader, TFile};
    use tkey::TKeyHeader;
    use compression::decompress;

    #[test]
    fn test_read_streaming_info() {
        let path = "/Users/kylelau519/Programming/rusty_root/rusty_root_io/testfiles/wzqcd_mc20a.root";
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
        dbg!(&tfile.streamer_info.streamer_info_header);
        let header = &tfile.streamer_info.streamer_info_header;
        assert_eq!(header.n_bytes, header.compressed_data.len() as u32 + header.key_len as u32);
        let compression_level = tfile.header.f_compress;
        let data = decompress(&header.compressed_data, compression_level);
        assert!(data.is_ok());
        assert!(data.unwrap().len() == header.obj_len as usize);
    }

    use crate::tlist::TList;
    #[test]
    fn test_read_root_file() {
        let path = "/Users/kylelau519/Programming/rusty_root/rusty_root_io/testfiles/output.root";
        let mut tfile = TFile::open(path).expect("Failed to open ROOT file");
        let header = &mut tfile.streamer_info.streamer_info_header;
        header.decompress_and_store(tfile.header.f_compress).expect("Failed to decompress streamer info");
        let tlist = TList::new_from_data(header.decompressed_data().unwrap());
        assert!(tlist.is_ok());
        dbg!(&tlist.unwrap());
    }
    #[test]
    fn test_read_first_envelope() {
        let path = "/Users/kylelau519/Programming/rusty_root/rusty_root_io/testfiles/output.root";
        let mut tfile = TFile::open(path).expect("Failed to open ROOT file");
        let header = &mut tfile.streamer_info.streamer_info_header;
        header.decompress_and_store(tfile.header.f_compress).expect("Failed to decompress streamer info");
        let tlist = TList::new_from_data(header.decompressed_data().unwrap());
        assert!(tlist.is_ok());
        let tlist = tlist.unwrap();
        let envelope = tlist.extract_first_envelope().expect("Failed to extract first envelope");
        dbg!(&envelope);
    }

    #[test]
    fn test_dump_streamer_info_ascii() {
        let path = "/Users/kylelau519/Programming/rusty_root/rusty_root_io/testfiles/wzqcd_mc20a.root";
        let mut tfile = TFile::open(path).expect("Failed to open ROOT file");
        let header = &mut tfile.streamer_info.streamer_info_header;
        header
            .decompress_and_store(tfile.header.f_compress)
            .expect("Failed to decompress streamer info");
        let data = header
            .decompressed_data()
            .expect("No decompressed data stored");
        dbg!(header);
        for i in 0..15 {
            let start = i * 100;
            let end = ((i + 1) * 100).min(data.len());
            let ascii: String = data.iter()
            .skip(start)
            .take(end - start)
            .map(|&b| if b.is_ascii_graphic() || b == b'\n' || b == b'\r' || b == b'\t' || b == b' ' { format!("{}", b as char) } else { format!("[{:02X}]", b) })
            .collect();
            println!("\n{}", ascii);
        }
    }

}
