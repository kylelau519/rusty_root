use crate::tkey::TKey;
use binrw::binread;
use binrw::BinRead;

/*
// https://root.cern/doc/v638/keyslist.html
*/
#[binread]
#[br(big)]
#[derive(Debug, Default)]
pub struct KeyList {
    key: TKey,
    n_keys: u32,
    #[br(count = n_keys)]
    keys: Vec<TKey>,
}

impl KeyList {
    pub fn read_from<R: binrw::io::Read + binrw::io::Seek>(
        reader: &mut R,
        offset: u64,
    ) -> binrw::BinResult<Self> {
        reader.seek(binrw::io::SeekFrom::Start(offset))?;
        Self::read_be(reader)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::io::BufReader;
    #[test]
    fn test_read_keylist() {
        let path =
            "/Users/kylelau519/Programming/rusty_root/rusty_root_io/testfiles/wzqcd_mc20a.root";
        let key_list_offset = 80365942;
        let file = File::open(path).expect("Failed to open ROOT file");
        let mut reader = BufReader::new(file);
        let key_list = KeyList::read_from(&mut reader, key_list_offset)
            .expect("Failed to read KeyList at offset");
        dbg!(&key_list);
    }
}
