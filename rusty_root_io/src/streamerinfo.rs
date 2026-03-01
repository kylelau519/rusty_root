use crate::constant::{K_BYTECOUNTMASK, K_NEWCLASSTAG};
use crate::tkey::TKey;
use crate::tlist::TList;
use crate::tstreamerinfo::TStreamerInfo;
use byteorder::ReadBytesExt;
use std::fs::File;
use std::io;
use std::io::{BufReader, Seek, SeekFrom};

// https://root.cern/doc/v638/streamerinfo.html
#[derive(Default)]
pub struct StreamerInfo {
    pub streamer_info_header: TKey,
    pub tlist: TList<TStreamerInfo>,
}

impl StreamerInfo {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tkey::TKey;
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
}
