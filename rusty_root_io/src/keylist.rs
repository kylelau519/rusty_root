use crate::tkey::TKey;
use byteorder::ReadBytesExt;
use std::fs::File;
use std::io::{BufReader, Seek, SeekFrom};

// https://root.cern/doc/v638/keyslist.html
/*
*  ----------TKey---------------
 byte 0->3  Nbytes    = Number of bytes in compressed record (TKey+data)              TKey::fNbytes
      4->5  Version   = TKey class version identifier                                 TKey::fVersion
      6->9  ObjLen    = Number of bytes of uncompressed data                          TKey::fObjLen
     10->13 Datime    = Date and time when record was written to file                 TKey::fDatime
                      | (year-1995)<<26|month<<22|day<<17|hour<<12|minute<<6|second
     14->15 KeyLen    = Number of bytes in the key structure (TKey)                   TKey::fKeyLen
     16->17 Cycle     = Cycle of key                                                  TKey::fCycle
     18->21 SeekKey   = Byte offset of record itself (consistency check)              TKey::fSeekKey
     22->25 SeekPdir  = Byte offset of parent directory record (directory)            TKey::fSeekPdir
     26->26 lname     = Number of bytes in the class name (5 or 10)                   TKey::fClassName
     27->.. ClassName = Object Class Name ("TFile" or "TDirectory")                   TKey::fClassName
      0->0  lname     = Number of bytes in the object name                            TNamed::fName
      1->.. Name      = lName bytes with the name of the object `<directory-name>`    TNamed::fName
      0->0  lTitle    = Number of bytes in the object title                           TNamed::fTitle
      1->.. Title     = lTitle bytes with the title of the object `<directory-title>` TNamed::fTitle
----------DATA---------------
      0->3  NKeys     = Number of keys in list (i.e. records in directory (non-recursive))
                      | Excluded:: The directory itself, KeysList, StreamerInfo, and FreeSegments
      4->.. TKey      = Sequentially for each record in directory,
                      |  the entire TKey portion of each record is replicated.
                      |  Note that SeekKey locates the record.
*/
#[derive(Debug, Default)]
pub struct KeyList {
    key: TKey,
    n_keys: u32,
    keys: Vec<TKey>,
}

impl KeyList {
    pub fn read_keylist_at(reader: &mut BufReader<File>, offset: u64) -> std::io::Result<Self> {
        let key = TKey::read_tkey_at(reader, offset)?;
        let n_keys = reader.read_u32::<byteorder::BigEndian>()?;
        let mut keys = Vec::with_capacity(n_keys as usize);
        for _ in 0..n_keys {
            let key = TKey::read_tkey(reader)?;
            keys.push(key);
        }
        Ok(Self { key, n_keys, keys })
    }
    pub fn read_keylist(reader: &mut BufReader<File>) -> std::io::Result<Self> {
        let loc = reader.seek(SeekFrom::Current(0))?;
        Self::read_keylist_at(reader, loc)
    }
}
