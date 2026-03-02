use crate::constant::K_BYTECOUNTMASK;
use crate::tnamed::TNamed;
use crate::tobjarray::TObjArray;
use crate::tstreamer_element::TStreamerElement;
use crate::utils::ClassInfo;
use binrw::binread;
use byteorder::ReadBytesExt;
use std::io::{Read, Seek, SeekFrom};

#[binread]
#[br(big)]
#[derive(Default, Debug)]
pub struct TStreamerInfo {
    #[br(map = |x: u32| x & K_BYTECOUNTMASK)]
    pub byte_count: u32,
    pub class_info: ClassInfo,
    #[br(map = |x: u32| x & K_BYTECOUNTMASK)]
    pub remaining_bytes: u32,
    pub version: u16,
    pub tnamed: TNamed,
    pub f_checksum: u32,
    pub f_class_version: u32,
    pub tobjarray: TObjArray<TStreamerElement>,
}

impl TStreamerInfo {
    pub fn read_tstreamer_info_at<R: Read + Seek>(
        reader: &mut R,
        offset: u64,
    ) -> std::io::Result<Self> {
        reader.seek(std::io::SeekFrom::Start(offset))?;
        let byte_count = reader.read_u32::<byteorder::BigEndian>()? & K_BYTECOUNTMASK;
        let class_info = ClassInfo::read_class_info(reader)?;
        let remaining_bytes = reader.read_u32::<byteorder::BigEndian>()? & K_BYTECOUNTMASK;
        let version = reader.read_u16::<byteorder::BigEndian>()?;
        let tnamed = TNamed::read_tnamed(reader)?;
        let f_checksum = reader.read_u32::<byteorder::BigEndian>()?;
        let f_class_version = reader.read_u32::<byteorder::BigEndian>()?;
        let tobjarray = TObjArray::<TStreamerElement>::read_tobjarray(reader)?;
        Ok(Self {
            byte_count,
            class_info,
            remaining_bytes,
            version,
            tnamed,
            f_checksum,
            f_class_version,
            tobjarray,
        })
    }

    pub fn read_tstreamer_info<R: Read + Seek>(reader: &mut R) -> std::io::Result<Self> {
        let loc = reader.seek(SeekFrom::Current(0))?;
        Self::read_tstreamer_info_at(reader, loc)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tkey::TKey;
    use crate::tlist::TList;
    use std::fs::File;

    #[test]
    fn test_read_streamer_info() {
        let path =
            "/Users/kylelau519/Programming/rusty_root/rusty_root_io/testfiles/streamer_info.bin";
        let file = File::open(path).expect("Failed to open streamer info file");
        let mut reader = std::io::BufReader::new(file);
        let tkey: TKey = TKey::read_tkey(&mut reader).expect("Failed to read TKey");
        dbg!(&tkey);
        let tlist: TList<()> =
            TList::read_tlist_metadata(&mut reader).expect("Failed to read TList");
        dbg!(&tlist);
        let mut tstreamers_info: Vec<TStreamerInfo> = Vec::new();
        for i in 0..1 {
            let tstreamer_info = TStreamerInfo::read_tstreamer_info(&mut reader)
                .expect("Failed to read TStreamerInfo");
            dbg!(&tstreamer_info);
            tstreamers_info.push(tstreamer_info);
        }
        dbg!(tstreamers_info);
    }
    use binrw::BinRead;
    #[test]
    fn test_read_streamer_info_binrw() {
        let path =
            "/Users/kylelau519/Programming/rusty_root/rusty_root_io/testfiles/streamer_info.bin";
        let file = File::open(path).expect("Failed to open streamer info file");
        let mut reader = std::io::BufReader::new(file);

        let tkey: TKey = TKey::read_be(&mut reader).expect("Failed to read TKey with BinRead");
        dbg!(&tkey);
        let tlist: TList<()> =
            TList::read_tlist_metadata(&mut reader).expect("Failed to read TList with BinRead");
        dbg!(&tlist);
        let tstreamer_info: TStreamerInfo =
            TStreamerInfo::read_be(&mut reader).expect("Failed to read TStreamerInfo with BinRead");
        dbg!(&tstreamer_info);
        // let mut tstreamers_info: Vec<TStreamerInfo> = Vec::new();
    }
}
