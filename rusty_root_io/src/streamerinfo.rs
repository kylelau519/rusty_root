use crate::tkey::TKey;
use crate::tlist::TList;
use crate::tstreamerinfo::TStreamerInfo;
use binrw::io::{Read, Seek, SeekFrom};
use binrw::BinRead;

// https://root.cern/doc/v638/streamerinfo.html
#[binrw::binread]
#[derive(Debug, Default)]
pub struct StreamerInfo {
    pub streamer_info_header: TKey,
    // the last element of StreamerInfo is not TStreamerInfo but some object called "ListOfRules".. not written anywhere, we just skip for the sanity
    #[br(parse_with = TKey::read_from_payload, args(&streamer_info_header))]
    pub tlist: TList<TStreamerInfo>,
}

impl StreamerInfo {
    pub fn read_from<R: Read + Seek>(reader: &mut R, offset: u64) -> binrw::BinResult<Self> {
        reader.seek(SeekFrom::Start(offset))?;
        Self::read_options(reader, binrw::Endian::Big, ())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tkey::TKey;
    use std::fs::File;
    use std::io::{BufReader, Read, Seek, SeekFrom};
    #[test]
    fn test_streamer_info_header() {
        let path =
            "/Users/kylelau519/Programming/rusty_root/rusty_root_io/testfiles/wzqcd_mc20a.root";
        let streamer_info_offset = 80357582;
        let file = File::open(path).expect("Failed to open ROOT file");
        let mut reader = BufReader::new(file);
        let key = TKey::read_from(&mut reader, streamer_info_offset)
            .expect("Failed to read TKey at offset");
        assert_eq!(key.title, "Doubly linked list");
        dbg!(&key);
    }

    use crate::compression::CompressionAlgorithm;
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
        let decompressed_data =
            CompressionAlgorithm::decompress(&data).expect("Failed to decompress data");
        dbg!(&decompressed_data.len());

        assert_eq!(decompressed_data.len(), obj_len as usize);
    }

    use crate::tlist::TList;
    use binrw::BinRead;
    #[test]
    fn test_read_all_streamer_info() {
        let path =
            "/Users/kylelau519/Programming/rusty_root/rusty_root_io/testfiles/wzqcd_mc20a.root";
        let streamer_info_offset = 80357582;
        let file = File::open(path).expect("Failed to open ROOT file");
        let mut reader = BufReader::new(file);

        let key = TKey::read_from(&mut reader, streamer_info_offset)
            .expect("Failed to read TKey at offset");
        assert_eq!(key.title, "Doubly linked list");
        let payload_len = (key.n_bytes - key.key_len as u32) as usize;

        let key_data = {
            let mut buf = vec![0u8; key.key_len as usize];
            reader
                .seek(SeekFrom::Start(streamer_info_offset))
                .expect("Failed to seek to TKey");
            reader
                .read_exact(&mut buf)
                .expect("Failed to read TKey data");
            buf
        };
        let compressed_data = {
            let mut buf = vec![0u8; payload_len];
            reader
                .seek(SeekFrom::Start(streamer_info_offset + key.key_len as u64))
                .expect("Failed to seek to compressed data");
            reader
                .read_exact(&mut buf)
                .expect("Failed to read compressed data");
            buf
        };
        // i forgot why it is Necessary to combine and read the key and decompressed together
        // my guess is there's some decompressed data point back to the tkey
        let decompressed_data =
            CompressionAlgorithm::decompress(&compressed_data).expect("Failed to decompress data");
        let mut combined_data = key_data;
        combined_data.extend_from_slice(&decompressed_data);
        let mut cursor = std::io::Cursor::new(combined_data);
        let _tkey: TKey =
            TKey::read_be(&mut cursor).expect("Failed to read TKey from combined data");
        let _tlist: TList<TStreamerInfo> =
            TList::read_be(&mut cursor).expect("Failed to read TList from decompressed data");
        dbg!(&_tlist);
    }
    #[test]
    fn test_read_streamer_with_tkey() {
        let path =
            "/Users/kylelau519/Programming/rusty_root/rusty_root_io/testfiles/wzqcd_mc20a.root";
        let streamer_info_offset = 80357582;
        let file = File::open(path).expect("Failed to open ROOT file");
        let mut reader = BufReader::new(file);

        let tkey = TKey::read_from(&mut reader, streamer_info_offset)
            .expect("Failed to read TKey at offset");
        dbg!(&tkey);

        let mut decompressed_cursor = tkey
            .decompress_full(&mut reader)
            .expect("Failed to decompress StreamerInfo data");
        decompressed_cursor
            .seek(SeekFrom::Start(tkey.key_len as u64))
            .expect("Failed to seek to decompressed data after TKey");
        let tlist = TList::<TStreamerInfo>::read_be(&mut decompressed_cursor)
            .expect("Failed to read TList of TStreamerInfo from decompressed data");
        dbg!(&tlist);
    }
}
