use crate::tkey::TKey;
use crate::utils;
use crate::utils::ReaderDynWidth;
use byteorder::{BigEndian, ReadBytesExt};
use std::fs::File;
use std::io;
use std::io::{BufReader, Seek, SeekFrom};

#[derive(Default, Debug)]
pub struct FirstRecordDict {
    pub key: TKey,
    pub data: FirstRecordData,
}

impl FirstRecordDict {
    pub fn read_first_record_dict<R: std::io::Read + std::io::Seek>(reader: &mut R, offset: u64) -> io::Result<Self> {
        let key = TKey::read_tkey_at(reader, offset)?;
        let data = FirstRecordData::read_header_dict_data(reader)?;
        Ok(Self { key, data })
    }
}

/*
 * ---------- DATA ----------
 * Byte Range      Member Name      Description
 * 0...0           lname           Number of bytes in the TFile name (TNamed::fName)
 * 1...            Name            lName bytes with the name of the TFile <file-name> (TNamed::fName)
 * 0...0           lTitle          Number of bytes in the TFile title (TNamed::fTitle)
 * 1...            Title           lTitle bytes with the title of the TFile <file-title> (TNamed::fTitle)
 * 0...1           Version         TDirectory class version identifier (TDirectory::Class_Version())
 * 2...5           DatimeC         Date and time when directory was created (TDirectory::fDatimeC)
 *                                | (year-1995)<<26 | month<<22 | day<<17 | hour<<12 | minute<<6 | second
 * 6...9           DatimeM         Date and time when directory was last modified (TDirectory::fDatimeM)
 *                                | (year-1995)<<26 | month<<22 | day<<17 | hour<<12 | minute<<6 | second
 * 10...13         NbytesKeys      Number of bytes in the associated KeysList record (TDirectory::fNbyteskeys)
 * 14...17         NbytesName      Number of bytes in TKey+TNamed at creation (TDirectory::fNbytesName)
 * 18...21 [18...25] SeekDir       Byte offset of directory record in file (64) (TDirectory::fSeekDir)
 * 22...25 [26...33] SeekParent    Byte offset of parent directory record in file (0) (TDirectory::fSeekParent)
 * 26...29 [34...41] SeekKeys      Byte offset of associated KeysList record in file (TDirectory::fSeekKeys)
 */
#[derive(Default, Debug)]
pub struct FirstRecordData {
    pub l_name: u8,
    pub name: String,
    pub l_title: u8,
    pub title: String,
    pub version: u16,
    pub datime_c: u32,
    pub datime_m: u32,
    pub n_bytes_keys: u32,
    pub n_bytes_name: u32,
    pub seek_dir: u64,
    pub seek_parent: u64,
    pub seek_keys: u64,
}

impl FirstRecordData {
    pub fn read_header_dict_data<R: std::io::Read + std::io::Seek>(reader: &mut R) -> io::Result<Self> {
        let loc = reader.seek(SeekFrom::Current(0))?;
        Self::read_header_dict_data_at(reader, loc)
    }

    pub fn read_header_dict_data_at<R: std::io::Read + std::io::Seek>(reader: &mut R, offset: u64) -> io::Result<Self> {
        reader.seek(SeekFrom::Start(offset))?;
        let l_name = utils::read_u1(reader)?;
        let name = utils::read_string(reader, l_name as usize)?;
        let l_title = utils::read_u1(reader)?;
        let title = utils::read_string(reader, l_title as usize)?;
        let version = reader.read_u16::<BigEndian>()?;
        let datime_c = reader.read_u32::<BigEndian>()?;
        let datime_m = reader.read_u32::<BigEndian>()?;
        let n_bytes_keys = reader.read_u32::<BigEndian>()?;
        let n_bytes_name = reader.read_u32::<BigEndian>()?;

        let reader_dyn_width = ReaderDynWidth::from_tkey_version(version); // TDirectory uses version 1000 for 64-bit offsets
        let seek_dir = reader_dyn_width.read_ptr(reader)?;
        let seek_parent = reader_dyn_width.read_ptr(reader)?;
        let seek_keys = reader_dyn_width.read_ptr(reader)?;

        Ok(Self {
            l_name,
            name,
            l_title,
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
    #[test]
    fn test_read_first_data_record_key() {
        let path =
            "/Users/kylelau519/Programming/rusty_root/rusty_root_io/testfiles/wzqcd_mc20a.root";
        let file = File::open(path).expect("Failed to open ROOT file");
        let mut reader = BufReader::new(file);
        let tkey_offset = 100u64;
        let key =
            TKey::read_tkey_at(&mut reader, tkey_offset).expect("Failed to read TKey at offset");
        assert_eq!(key.name, "user.holau.700590.Sh_2212_llvvjj_ss.e8433_s3681_r13167_r13146_p6697.46550259._000001.output.root");
        dbg!(&key);
    }

    #[test]
    fn test_read_first_data_record_data() {
        let path =
            "/Users/kylelau519/Programming/rusty_root/rusty_root_io/testfiles/wzqcd_mc20a.root";
        let file = File::open(path).expect("Failed to open ROOT file");
        let mut reader = BufReader::new(file);
        let begin = 100u64;
        let f_units = 4u8;
        dbg!(begin, f_units);
        let first_data_key =
            TKey::read_tkey_at(&mut reader, begin).expect("Failed to read TKey at offset");
        let first_data_data = FirstRecordData::read_header_dict_data(&mut reader)
            .expect("Failed to read header dict data at offset");
        assert_eq!(decode_datime(first_data_key.datime), "2025-09-27 06:16:14");
        assert_eq!(
            decode_datime(first_data_data.datime_m),
            "2025-09-27 06:16:17"
        );
        dbg!(&first_data_key);
        dbg!(&first_data_data);
    }
}
