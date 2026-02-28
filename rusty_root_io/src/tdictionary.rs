use std::io::{BufReader, Read, Seek};

use crate::tkey::TKey;
use crate::utils::ReaderDynWidth;
use byteorder::ReadBytesExt;
use std::fs::File;

/*
* ----------TKey--------------
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
 --------DATA----------------
       0->1           Version   = TDirectory class version identifier                           TDirectory::Class_Version()
       2->5           DatimeC   = Date and time when directory was created                      TDirectory::fDatimeC
                                | (year-1995)<<26|month<<22|day<<17|hour<<12|minute<<6|second
       6->9           DatimeM   = Date and time when directory was last modified                TDirectory::fDatimeM
                                | (year-1995)<<26|month<<22|day<<17|hour<<12|minute<<6|second
      10->13          NbytesKeys= Number of bytes in the associated KeysList record             TDirectory::fNbyteskeys
      14->17          NbytesName= Number of bytes in TKey+TNamed at creation                    TDirectory::fNbytesName
      18->21 [18->25] SeekDir   = Byte offset of directory record in file                       TDirectory::fSeekDir
      22->25 [26->33] SeekParent= Byte offset of parent directory record in file                TDirectory::fSeekParent
      26->29 [34->41] SeekKeys  = Byte offset of associated KeysList record in file             TDirectory::fSeekKeys
      30->31 [42->43] UUID vers = TUUID class version identifier                                TUUID::Class_Version()
      32->47 [44->59] UUID      = Universally Unique Identifier                                 TUUID::fTimeLow through fNode[6]
      48->59          Extra space to allow SeekKeys to become 64 bit without moving this header
*/
#[derive(Debug)]
pub struct TDictData {
    version: u16,
    datime_c: u32,
    datime_m: u32,
    n_bytes_keys: u32,
    n_bytes_name: u32,
    seek_dir: u64,
    seek_parent: u64,
    seek_keys: u64,
    uuid_vers: u16,
    uuid: [u8; 16],
}

#[derive(Debug)]
pub struct TDictionary {
    tkey: TKey,
    data: TDictData,
}

impl TDictData {
    pub fn read_dict_data_at(reader: &mut BufReader<File>, offset: u64) -> std::io::Result<Self> {
        reader.seek(std::io::SeekFrom::Start(offset))?;
        let version = reader.read_u16::<byteorder::BigEndian>()?;
        let reader_dyn_width = ReaderDynWidth::from_tkey_version(version); // TDirectory uses version 1000 for 64-bit offsets
        let datime_c = reader.read_u32::<byteorder::BigEndian>()?;
        let datime_m = reader.read_u32::<byteorder::BigEndian>()?;
        let n_bytes_keys = reader.read_u32::<byteorder::BigEndian>()?;
        let n_bytes_name = reader.read_u32::<byteorder::BigEndian>()?;
        let seek_dir = reader_dyn_width.read_ptr(reader)?;
        let seek_parent = reader_dyn_width.read_ptr(reader)?;
        let seek_keys = reader_dyn_width.read_ptr(reader)?;
        let uuid_vers = reader.read_u16::<byteorder::BigEndian>()?;
        let mut uuid = [0u8; 16];
        reader.read_exact(&mut uuid)?;
        match reader_dyn_width {
            ReaderDynWidth::Off32 => {
                let mut skip_buf = [0u8; 12];
                reader.read_exact(&mut skip_buf)?;
            }
            ReaderDynWidth::Off64 => {}
        }
        Ok(Self {
            version,
            datime_c,
            datime_m,
            n_bytes_keys,
            n_bytes_name,
            seek_dir,
            seek_parent,
            seek_keys,
            uuid_vers,
            uuid,
        })
    }

    pub fn read_dict_data(reader: &mut BufReader<File>) -> std::io::Result<Self> {
        let loc = reader.seek(std::io::SeekFrom::Current(0))?;
        Self::read_dict_data_at(reader, loc)
    }
}

impl TDictionary {
    pub fn read_tdict_at(reader: &mut BufReader<File>, offset: u64) -> std::io::Result<Self> {
        let tkey = TKey::read_tkey_at(reader, offset)?;
        let data = TDictData::read_dict_data(reader)?;
        Ok(Self { tkey, data })
    }

    pub fn read_tdict(reader: &mut BufReader<File>) -> std::io::Result<Self> {
        let loc = reader.seek(std::io::SeekFrom::Current(0))?;
        Self::read_tdict_at(reader, loc)
    }
}
