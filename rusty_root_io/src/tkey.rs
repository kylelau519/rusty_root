use crate::utils::{self, binrw_read_string, ReaderDynWidth};
use binrw::{BinRead, BinReaderExt, BinResult, Endian};
use byteorder::{BigEndian, ReadBytesExt};
use std::fmt;
use std::io;
use std::io::{Read, Seek, SeekFrom};

/*
 * https://root.cern/doc/v638/tdirectory.html
----------TKey--------------
 byte 0->3           Nbytes    = Number of bytes in compressed record (Tkey+data)              TKey::fNbytes
      4->5           Version   = TKey class version identifier                                 TKey::fVersion
      6->9           ObjLen    = Number of bytes of uncompressed data                          TKey::fObjLen
     10->13          Datime    = Date and time when record was written to file                 TKey::fDatime
                               | (year-1995)<<26|month<<22|day<<17|hour<<12|minute<<6|second
     14->15          KeyLen    = Number of bytes in key structure (TKey)                       TKey::fKeyLen
     16->17          Cycle     = Cycle of key                                                  TKey::fCycle
     18->21 [18->25] SeekKey   = Byte offset of record itself (consistency check)              TKey::fSeekKey
     22->25 [26->33] SeekPdir  = Byte offset of parent directory record                        TKey::fSeekPdir
     26->26 [33->33] lname     = Number of bytes in the class name (10)                        TKey::fClassName
     27->.. [34->..] ClassName = Object Class Name ("TDirectory")                              TKey::fClassName
      0->0           lname     = Number of bytes in the object name                            TNamed::fName
      1->..          Name      = lName bytes with the name of the object `<directory-name>`    TNamed::fName
      0->0           lTitle    = Number of bytes in the object title                           TNamed::fTitle
      1->..          Title     = lTitle bytes with the title of the object `<directory-title>` TNamed::fTitle
 */

#[derive(Default)]
pub struct TKey {
    pub n_bytes: u32,
    pub version: u16,
    pub obj_len: u32,
    pub datime: u32,
    pub key_len: u16,
    pub cycle: u16,
    pub seek_key: u64,
    pub seek_p_dir: u64,
    pub l_class_name: u8,
    pub class_name: String,
    pub l_name: u8,
    pub name: String,
    pub l_title: u8,
    pub title: String,
}
impl TKey {
    pub fn new() -> Self {
        TKey {
            n_bytes: 0,
            version: 0,
            obj_len: 0,
            datime: 0,
            key_len: 0,
            cycle: 0,
            seek_key: 0,
            seek_p_dir: 0,
            l_class_name: 0,
            class_name: String::new(),
            l_name: 0,
            name: String::new(),
            l_title: 0,
            title: String::new(),
        }
    }
    pub fn read_tkey_at<R: Read + Seek>(reader: &mut R, offset: u64) -> io::Result<Self> {
        reader.seek(SeekFrom::Start(offset))?;
        let n_bytes = reader.read_u32::<BigEndian>()?;
        let version = reader.read_u16::<BigEndian>()?;
        let obj_len = reader.read_u32::<BigEndian>()?;
        let datime = reader.read_u32::<BigEndian>()?;
        let key_len = reader.read_u16::<BigEndian>()?;
        let cycle = reader.read_u16::<BigEndian>()?;
        let reader_dyn_width = ReaderDynWidth::from_tkey_version(version);
        let seek_key = reader_dyn_width.read_ptr(reader)?;
        let seek_p_dir = reader_dyn_width.read_ptr(reader)?;
        let l_class_name = reader.read_u8()?;
        let class_name = utils::read_string(reader, l_class_name as usize)?;
        let l_name = reader.read_u8()?;
        let name = utils::read_string(reader, l_name as usize)?;
        let l_title = reader.read_u8()?;
        let title = utils::read_string(reader, l_title as usize)?;
        let key = TKey {
            n_bytes,
            version,
            obj_len,
            datime,
            key_len,
            cycle,
            seek_key,
            seek_p_dir,
            l_class_name,
            class_name,
            l_name,
            name,
            l_title,
            title,
        };
        Ok(key)
    }

    pub fn read_tkey<R: Read + Seek>(reader: &mut R) -> io::Result<Self> {
        let loc = reader.seek(SeekFrom::Current(0))?;
        TKey::read_tkey_at(reader, loc)
    }
}
impl fmt::Debug for TKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("TKey")
            .field("n_bytes", &self.n_bytes)
            .field("version", &self.version)
            .field("obj_len", &self.obj_len)
            .field("datime", &self.datime)
            .field("key_len", &self.key_len)
            .field("cycle", &self.cycle)
            .field("seek_key", &self.seek_key)
            .field("seek_p_dir", &self.seek_p_dir)
            .field("l_class_name", &self.l_class_name)
            .field("class_name", &self.class_name)
            .field("l_name", &self.l_name)
            .field("name", &self.name)
            .field("l_title", &self.l_title)
            .field("title", &self.title)
            .finish()
    }
}

impl BinRead for TKey {
    type Args<'a> = ();

    fn read_options<R: Read + Seek>(
        reader: &mut R,
        endian: Endian,
        _args: Self::Args<'_>,
    ) -> BinResult<Self> {
        // 1. Read the fixed-width fields first
        let n_bytes: u32 = reader.read_type(endian)?;
        let version: u16 = reader.read_type(endian)?;
        let obj_len: u32 = reader.read_type(endian)?;
        let datime: u32 = reader.read_type(endian)?;
        let key_len: u16 = reader.read_type(endian)?;
        let cycle: u16 = reader.read_type(endian)?;

        // 2. THE FIX: Handle variable width pointers
        // ROOT Logic: if version > 1000, pointers are 64-bit
        let (seek_key, seek_p_dir) = if version > 1000 {
            let s_key: u64 = reader.read_type(endian)?;
            let s_pdir: u64 = reader.read_type(endian)?;
            (s_key, s_pdir)
        } else {
            let s_key: u32 = reader.read_type(endian)?;
            let s_pdir: u32 = reader.read_type(endian)?;
            (s_key as u64, s_pdir as u64) // Cast to u64 for struct parity
        };

        // 3. Read Strings using our parse_with logic or helper
        let l_class_name: u8 = reader.read_type(endian)?;
        let class_name = binrw_read_string(reader, endian, (l_class_name,))?;

        let l_name: u8 = reader.read_u8()?;
        let name = binrw_read_string(reader, endian, (l_name,))?;

        let l_title: u8 = reader.read_u8()?;
        let title = binrw_read_string(reader, endian, (l_title,))?;

        Ok(TKey {
            n_bytes,
            version,
            obj_len,
            datime,
            key_len,
            cycle,
            seek_key,
            seek_p_dir,
            l_class_name,
            class_name,
            l_name,
            name,
            l_title,
            title,
        })
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::decode_datime;
    #[test]
    fn test_decode_datime() {
        let mut key = TKey::new();
        key.datime = 2054579214;
        assert_eq!(decode_datime(key.datime), "2025-09-27 06:16:14");
    }

    use binrw::BinRead;
    use std::fs::File;
    use std::io::SeekFrom;

    #[test]
    fn test_read_key_with_binrw() {
        let path =
            "/Users/kylelau519/Programming/rusty_root/rusty_root_io/testfiles/wzqcd_mc20a.root";
        let key_list_offset = 80365942;
        let file = File::open(path).expect("Failed to open ROOT file");
        let mut reader = std::io::BufReader::new(file);
        reader
            .seek(SeekFrom::Start(key_list_offset))
            .expect("Failed to seek to key list offset");

        let key = TKey::read_be(&mut reader).expect("Failed to read TKey with BinRead");
        dbg!(&key);
        assert_eq!(key.class_name, "TFile");
        assert_eq!(key.name, "user.holau.700590.Sh_2212_llvvjj_ss.e8433_s3681_r13167_r13146_p6697.46550259._000001.output.root");
    }
}
