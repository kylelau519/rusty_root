use crate::constant::K_BYTECOUNTMASK;
use crate::tnamed::TNamed;
use crate::tobjarray::TObjArray;
use crate::tstreamer_element::TStreamerElement;
use crate::utils::ClassInfo;
use binrw::{BinRead, BinReaderExt, BinResult, Endian};
use std::io::{Read, Seek, SeekFrom};

#[derive(Default, Debug)]
pub struct TStreamerInfo {
    pub byte_count: u32,
    pub class_info: ClassInfo,
    pub remaining_bytes: u32,
    pub version: u16,
    pub tnamed: TNamed,
    pub f_checksum: u32,
    pub f_class_version: u32,
    pub tobjarray: TObjArray<TStreamerElement>,
}

impl BinRead for TStreamerInfo {
    type Args<'a> = ();

    fn read_options<R: Read + Seek>(
        reader: &mut R,
        endian: Endian,
        _args: Self::Args<'_>,
    ) -> BinResult<Self> {
        let start_pos = reader.stream_position()?;
        let byte_count = reader.read_type::<u32>(endian)? & K_BYTECOUNTMASK;
        let class_info = ClassInfo::read_options(reader, endian, ())?;

        if class_info.get_class_name() == "TStreamerInfo" {
            let remaining_bytes = reader.read_type::<u32>(endian)? & K_BYTECOUNTMASK;
            let version = reader.read_type::<u16>(endian)?;
            let tnamed = TNamed::read_options(reader, endian, ())?;
            let f_checksum = reader.read_type::<u32>(endian)?;
            let f_class_version = reader.read_type::<u32>(endian)?;
            let tobjarray = TObjArray::<TStreamerElement>::read_options(reader, endian, ())?;
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
        } else {
            // Skip the rest of the object
            reader.seek(SeekFrom::Start(start_pos + byte_count as u64 + 4))?;
            Ok(Self {
                byte_count,
                class_info,
                ..Default::default()
            })
        }
    }
}

impl TStreamerInfo {
    pub fn read_tstreamer_info_at<R: Read + Seek>(
        reader: &mut R,
        offset: u64,
    ) -> std::io::Result<Self> {
        reader.seek(std::io::SeekFrom::Start(offset))?;
        // Use the BinRead implementation for consistency
        Self::read_be(reader)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e.to_string()))
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
        let _tkey: TKey = TKey::read_tkey(&mut reader).expect("Failed to read TKey");
        let _tlist: TList<()> =
            TList::read_tlist_metadata(&mut reader).expect("Failed to read TList");
        let mut tstreamers_info: Vec<TStreamerInfo> = Vec::new();
        for _i in 0..1 {
            let tstreamer_info = TStreamerInfo::read_tstreamer_info(&mut reader)
                .expect("Failed to read TStreamerInfo");
            tstreamers_info.push(tstreamer_info);
        }
        assert_eq!(tstreamers_info.is_empty(), false);
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
        assert_eq!(tstreamer_info.f_checksum, 3753331260);
    }
}
