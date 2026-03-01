use crate::first_record::FirstRecordDict;
use crate::keylist::KeyList;
use crate::utils::ReaderDynWidth;
use byteorder::{BigEndian, ReadBytesExt};
use std::fs::File;
use std::io;
use std::io::{BufReader, Read, Seek};
use std::sync::Arc;

/*
    https://root.cern/doc/v638/header.html
    Byte Range      	Record Name     	Description
    0...3	            "root"	            Identifies this file as a ROOT file
    4...7	            Version         	File format version	TFile::fVersion (10000major+100minor+cycle (e.g. 62206 for 6.22.06))
    8...11	            BEGIN           	Byte offset of first data record (100)	TFile::fBEGIN
    12...15 [12...19]	END	                Pointer to first free word at the EOF	TFile::fEND (will be == to file size in bytes)
    16...19 [20...27]	SeekFree        	Byte offset of FreeSegments record	TFile::fSeekFree
    20...23 [28...31]	NbytesFree      	Number of bytes in FreeSegments record	TFile::fNBytesFree
    24...27 [32...35]	nfree           	Number of free data records
    28...31 [36...39]	NbytesName      	Number of bytes in TKey+TNamed for TFile at creation	TDirectory::fNbytesName
    32...32 [40...40]	Units	            Number of bytes for file pointers (4)	TFile::fUnits
    33...36 [41...44]	Compress        	Zip compression level (i.e. 0-9)	TFile::fCompress
    37...40 [45...52]	SeekInfo        	Byte offset of StreamerInfo record	TFile::fSeekInfo
    41...44 [53...56]	NbytesInfo      	Number of bytes in StreamerInfo record	TFile::fNbytesInfo
    45...46 [57...58]	UUID vers       	TUUID class version identifier	TUUID::Class_Version()
    47...62 [59...74]	UUID	            Universally Unique Identifier	TUUID::fTimeLow through fNode[6]
    63...99 [75...99]		                Extra space to allow END, SeekFree, or SeekInfo to become 64 bit without moving this header
*/

#[derive(Debug, Default)]
pub struct TFileHeader {
    _magic: [u8; 4],
    pub f_version: u32,
    pub f_begin: u32,
    pub f_end: u64,
    pub f_seek_free: u64,
    pub f_nbytes_free: u32,
    pub n_free: u32,
    pub f_nbytes_name: u32,
    pub f_units: u8,
    pub f_compress: i32,
    pub f_seek_info: u64,
    pub f_nbytes_info: u32,
    pub f_uuid_vers: u16,
    pub f_uuid: [u8; 16],
}

#[derive(Debug)]
pub struct TFile {
    reader: BufReader<File>,
    pub header: TFileHeader,
    pub first_data_record: FirstRecordDict,
    pub key_list: KeyList,
    pub contents: Arc<[u8]>,
    // pub streamer_info: StreamerInfo,
    // other fields...
}
impl TFile {
    pub fn open(path: &str) -> io::Result<Self> {
        let file = File::open(path)?;
        let mut reader = BufReader::new(file);
        let header = TFileHeader::read_header(&mut reader)?;
        let first_data_record =
            FirstRecordDict::read_first_record_dict(&mut reader, header.f_begin as u64)?;
        let key_list_offset = first_data_record.data.seek_keys;
        let key_list = KeyList::read_keylist_at(&mut reader, key_list_offset)?;
        let contents = Arc::new([]);
        Ok(TFile {
            reader,
            header,
            first_data_record,
            key_list,
            contents,
        })
    }

    pub fn reader_mut(&mut self) -> &mut BufReader<File> {
        &mut self.reader
    }
}

impl TFileHeader {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn read_header<R: Read + Seek>(reader: &mut R) -> io::Result<Self> {
        let _magic = Self::parse_magic(reader)?;
        let f_version = reader.read_u32::<BigEndian>()?;
        let f_begin = reader.read_u32::<BigEndian>()?;
        let reader_dyn_widht = ReaderDynWidth::from_tfile_version(f_version);
        // Read the rest of the header fields in the documented order
        let f_end = reader_dyn_widht.read_ptr(reader)?;
        let f_seek_free = reader_dyn_widht.read_ptr(reader)?;
        let f_nbytes_free = reader.read_u32::<BigEndian>()?;
        let n_free = reader.read_u32::<BigEndian>()?;
        let f_nbytes_name = reader.read_u32::<BigEndian>()?;
        let f_units = Self::parse_f_unit(reader)?;
        let f_compress = reader.read_i32::<BigEndian>()?;
        let f_seek_info = reader_dyn_widht.read_ptr(reader)?;
        let f_nbytes_info = reader.read_u32::<BigEndian>()?;
        let f_uuid_vers = reader.read_u16::<BigEndian>()?;
        let f_uuid = Self::parse_f_uuid(reader)?;

        let header = TFileHeader {
            _magic,
            f_version,
            f_begin,
            f_end,
            f_seek_free,
            f_nbytes_free,
            n_free,
            f_nbytes_name,
            f_units,
            f_compress,
            f_seek_info,
            f_nbytes_info,
            f_uuid_vers,
            f_uuid,
        };
        Ok(header)
    }
    fn parse_f_unit<R: std::io::Read + std::io::Seek>(reader: &mut R) -> io::Result<u8> {
        let mut units_buf = [0u8; 1];
        reader.read_exact(&mut units_buf)?;
        Ok(units_buf[0])
    }
    fn parse_f_uuid<R: std::io::Read + std::io::Seek>(reader: &mut R) -> io::Result<[u8; 16]> {
        let mut uuid_buf = [0u8; 16];
        reader.read_exact(&mut uuid_buf)?;
        Ok(uuid_buf)
    }
    fn parse_magic<R: std::io::Read + std::io::Seek>(reader: &mut R) -> io::Result<[u8; 4]> {
        let mut magic_buf = [0u8; 4];
        reader.read_exact(&mut magic_buf)?;
        if &magic_buf != b"root" {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "not a ROOT file",
            ));
        }
        Ok(magic_buf)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_read_root_header() {
        let path =
            "/Users/kylelau519/Programming/rusty_root/rusty_root_io/testfiles/wzqcd_mc20a.root";
        let file = File::open(path).expect("Failed to open ROOT file");
        let mut reader = BufReader::new(file);
        let header = match TFileHeader::read_header(&mut reader) {
            Ok(h) => h,
            Err(e) => panic!("Failed to read ROOT header: {:?}", e),
        };
        dbg!(&header);
    }
}
