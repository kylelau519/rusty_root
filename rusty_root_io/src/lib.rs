pub mod tkey;
pub mod tfile;
pub mod compression;

#[cfg(test)]
mod tests {
    use std::fs::File;

    use super::*;
    use tfile::{TFileHeader, TFile};
    use tkey::TKeyHeader;

    #[test]
    fn test_read_streaming_info() {
        let path = "/Users/kylelau519/Programming/rusty_root/rusty_root_io/testfiles/output.root";
        let mut tfile = TFile::open(path).expect("Failed to open ROOT file");
        let tkey_offset = tfile.header.f_seek_info;
        let f_units = tfile.header.f_units;
        let key = TKeyHeader::read_tkey_at(tfile.reader_mut(), tkey_offset, f_units).expect("Failed to read TKey at offset");
        assert!(key.class_name == "TList");
        assert!(key.name == "StreamerInfo");
    }

}
    
