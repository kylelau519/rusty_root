use crate::constant::K_BYTECOUNTMASK;
use crate::tnamed::TNamed;
use crate::tobjarray::TObjArray;
use crate::tstreamer_element::TStreamerElement;
use crate::utils::ClassInfo;
use byteorder::ReadBytesExt;
use std::io::{Read, Seek, SeekFrom};

#[derive(Default, Debug)]
pub struct TStreamerInfo {
    pub byte_count: u32,
    pub class_info: ClassInfo,
    pub remaining_bytes: u32,
    pub version: u16,
    pub tnamed: TNamed,
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
        let tobjarray = TObjArray::<TStreamerElement>::default();
        Ok(Self {
            byte_count,
            class_info,
            remaining_bytes,
            version,
            tnamed,
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
        for i in 0..3 {
            let tstreamer_info = TStreamerInfo::read_tstreamer_info(&mut reader)
                .expect("Failed to read TStreamerInfo");
            dbg!(&tstreamer_info);
            tstreamers_info.push(tstreamer_info);
        }
        dbg!(tstreamers_info);
    }
}
