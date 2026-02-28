use byteorder::{BigEndian, ReadBytesExt};
use std::fmt;
use std::fs::File;
use std::io;
use std::io::{BufReader, Read, Seek, SeekFrom};
use std::ops::Deref;
use std::sync::Arc;

use crate::compression::HasCompressedData;
use crate::utils;
use crate::utils::ReaderDynWidth;

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
    pub fn read_tkey_at(reader: &mut BufReader<File>, offset: u64) -> io::Result<Self> {
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
        let l_class_name = utils::read_u1(reader)?;
        let class_name = utils::read_string(reader, l_class_name as usize)?;
        let l_name = utils::read_u1(reader)?;
        let name = utils::read_string(reader, l_name as usize)?;
        let l_title = utils::read_u1(reader)?;
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

    pub fn read_tkey(reader: &mut BufReader<File>) -> io::Result<Self> {
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

#[derive(Default)]
pub struct TKeyHeader {
    pub base_key: TKey,
    pub compressed_data: Vec<u8>,
    pub decompressed_data: Option<Arc<[u8]>>,
}

impl fmt::Debug for TKeyHeader {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("TKeyHeader")
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
            .field(
                "compressed_data",
                &self
                    .compressed_data
                    .get(..10)
                    .unwrap_or(&self.compressed_data),
            )
            .field(
                "decompressed_data",
                &self
                    .decompressed_data
                    .as_ref()
                    .map(|v| v.get(..10).unwrap_or(&v[..])),
            )
            .finish()
    }
}
impl TKeyHeader {
    pub fn new() -> Self {
        TKeyHeader {
            base_key: TKey::new(),
            compressed_data: Vec::new(),
            decompressed_data: None,
        }
    }

    pub fn read_tkey_at(reader: &mut BufReader<File>, offset: u64) -> io::Result<Self> {
        let key = TKey::read_tkey_at(reader, offset)?;
        let keyheader = TKeyHeader {
            base_key: key,
            compressed_data: Vec::new(),
            decompressed_data: None,
        };
        Ok(keyheader)
    }

    pub fn read_tkey_at_save_payload(
        reader: &mut BufReader<File>,
        offset: u64,
    ) -> io::Result<Self> {
        let mut keyheader = Self::read_tkey_at(reader, offset)?;
        keyheader.compressed_data = keyheader.parse_payload(reader)?;
        Ok(keyheader)
    }

    fn parse_payload(&self, reader: &mut BufReader<File>) -> io::Result<Vec<u8>> {
        let payload_offset = self.seek_key + self.key_len as u64;
        reader.seek(SeekFrom::Start(payload_offset))?;
        let payload_buf = self.n_bytes - self.key_len as u32;
        let mut data_buf = vec![0u8; payload_buf as usize];
        reader.read_exact(&mut data_buf)?;
        Ok(data_buf)
    }
}
impl Deref for TKeyHeader {
    type Target = TKey;

    fn deref(&self) -> &Self::Target {
        &self.base_key
    }
}

impl HasCompressedData for TKeyHeader {
    fn get_compressed_data(&self) -> &[u8] {
        &self.compressed_data
    }

    fn get_compressed_len(&self) -> usize {
        self.compressed_data.len()
    }

    fn get_uncompressed_len(&self) -> usize {
        self.obj_len as usize
    }

    fn decompressed_data(&self) -> Option<Arc<[u8]>> {
        self.decompressed_data.clone()
    }

    fn decompressed_data_mut(&mut self) -> &mut Option<Arc<[u8]>> {
        &mut self.decompressed_data
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
}
