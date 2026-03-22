use crate::tkey::TKey;
use crate::tstring::TString;
use crate::utils::ReaderDynWidth;
use binrw::io::{Read, Seek};
use binrw::{binread, BinRead, BinReaderExt, BinResult, Endian};

#[binread]
#[derive(Default, Debug)]
pub struct FirstRecordDict {
    pub key: TKey,
    pub data: FirstRecordData,
}
impl FirstRecordDict {
    pub fn read_from<R: Read + Seek>(reader: &mut R, offset: u64) -> BinResult<Self> {
        reader.seek(std::io::SeekFrom::Start(offset))?;
        Self::read_be(reader)
    }
}
/*
 * https://root.cern/doc/v636/tfile.html
 */
#[derive(Default, Debug)]
pub struct FirstRecordData {
    pub name: TString,
    pub title: TString,
    pub version: u16,
    pub datime_c: u32,
    pub datime_m: u32,
    pub n_bytes_keys: u32,
    pub n_bytes_name: u32,
    pub seek_dir: u64,
    pub seek_parent: u64,
    pub seek_keys: u64,
}

impl BinRead for FirstRecordData {
    type Args<'a> = ();

    fn read_options<R: Read + Seek>(
        reader: &mut R,
        endian: Endian,
        _args: Self::Args<'_>,
    ) -> BinResult<Self> {
        let name = TString::read_options(reader, endian, ())?;
        let title = TString::read_options(reader, endian, ())?;
        let version = reader.read_type(endian)?;
        let datime_c = reader.read_type(endian)?;
        let datime_m = reader.read_type(endian)?;
        let n_bytes_keys = reader.read_type(endian)?;
        let n_bytes_name = reader.read_type(endian)?;

        let reader_dyn_width = ReaderDynWidth::from_tkey_version(version); // TDirectory uses version 1000 for 64-bit offsets
        let seek_dir = reader_dyn_width.read_ptr(reader)?;
        let seek_parent = reader_dyn_width.read_ptr(reader)?;
        let seek_keys = reader_dyn_width.read_ptr(reader)?;

        Ok(Self {
            name,
            title,
            version,
            datime_c,
            datime_m,
            n_bytes_keys,
            n_bytes_name,
            seek_dir,
            seek_parent,
            seek_keys,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::decode_datime;
    use std::fs::File;
    use std::io::BufReader;
    #[test]
    fn test_read_first_data_record_key() {
        let path =
            "/Users/kylelau519/Programming/rusty_root/rusty_root_io/testfiles/wzqcd_mc20a.root";
        let file = File::open(path).expect("Failed to open ROOT file");
        let mut reader = BufReader::new(file);
        let tkey_offset = 100u64;
        let key = TKey::read_from(&mut reader, tkey_offset).expect("Failed to read TKey at offset");
        assert_eq!(key.name, "user.holau.700590.Sh_2212_llvvjj_ss.e8433_s3681_r13167_r13146_p6697.46550259._000001.output.root");
    }

    #[test]
    fn test_read_first_data_record_data() {
        let path =
            "/Users/kylelau519/Programming/rusty_root/rusty_root_io/testfiles/wzqcd_mc20a.root";
        let file = File::open(path).expect("Failed to open ROOT file");
        let mut reader = BufReader::new(file);
        let begin = 100u64;
        let first_data_key =
            TKey::read_from(&mut reader, begin).expect("Failed to read TKey at offset");
        let first_data_data = FirstRecordData::read_options(&mut reader, Endian::Big, ())
            .expect("Failed to read header dict data at offset");
        assert_eq!(decode_datime(first_data_key.datime), "2025-09-27 06:16:14");
        assert_eq!(
            decode_datime(first_data_data.datime_m),
            "2025-09-27 06:16:17"
        );
    }
}
