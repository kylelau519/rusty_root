use crate::tkey::TKey;
use byteorder::{BigEndian, ReadBytesExt};
use std::fs::File;
use std::io;
use std::io::{BufReader, Read, Seek, SeekFrom};

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

pub struct TDictionary {
    tkey: TKey,
    data: TDictData,
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
pub struct TFileHeaderDictData {
    pub lname: u8,
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

impl TFileHeaderDictData {
    pub fn read_header_dict_data(reader: &mut BufReader<File>) -> io::Result<Self> {
        let lname = Self::parse_lname(reader)?;
        let name = Self::parse_string(reader, lname as usize)?;
        let l_title = Self::parse_lname(reader)?;
        let title = Self::parse_string(reader, l_title as usize)?;
        let version = reader.read_u16::<BigEndian>()?;
        let datime_c = reader.read_u32::<BigEndian>()?;
        let datime_m = reader.read_u32::<BigEndian>()?;
        let n_bytes_keys = reader.read_u32::<BigEndian>()?;
        let n_bytes_name = reader.read_u32::<BigEndian>()?;
        let seek_dir = reader.read_u64::<BigEndian>()?;
        let seek_parent = reader.read_u64::<BigEndian>()?;
        let seek_keys = reader.read_u64::<BigEndian>()?;

        Ok(Self {
            lname,
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

    pub fn read_header_dict_data_at(reader: &mut BufReader<File>, offset: u64) -> io::Result<Self> {
        reader.seek(SeekFrom::Start(offset))?;
        let lname = Self::parse_lname(reader)?;
        let name = Self::parse_string(reader, lname as usize)?;
        let l_title = Self::parse_lname(reader)?;
        let title = Self::parse_string(reader, l_title as usize)?;
        let version = reader.read_u16::<BigEndian>()?;
        let datime_c = reader.read_u32::<BigEndian>()?;
        let datime_m = reader.read_u32::<BigEndian>()?;
        let n_bytes_keys = reader.read_u32::<BigEndian>()?;
        let n_bytes_name = reader.read_u32::<BigEndian>()?;
        let seek_dir = reader.read_u64::<BigEndian>()?;
        let seek_parent = reader.read_u64::<BigEndian>()?;
        let seek_keys = reader.read_u64::<BigEndian>()?;

        Ok(Self {
            lname,
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

    fn parse_lname(reader: &mut BufReader<File>) -> io::Result<u8> {
        let mut lname_buf = [0u8; 1];
        reader.read_exact(&mut lname_buf)?;
        Ok(lname_buf[0])
    }

    fn parse_string(reader: &mut BufReader<File>, length: usize) -> io::Result<String> {
        let mut str_buf = vec![0u8; length];
        reader.read_exact(&mut str_buf)?;
        let s = String::from_utf8_lossy(&str_buf).to_string();
        Ok(s)
    }
}
