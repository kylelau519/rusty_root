use std::fs::File;
use std::io::{BufReader, Read, Seek, SeekFrom};
use std::io;
use byteorder::{BigEndian, ReadBytesExt};


/*
Byte Range      | Member Name | Description
----------------|-----------|--------------
1->4            | Nbytes    | Length of compressed object (in bytes)
5->6            | Version   | TKey version identifier
7->10           | ObjLen    | Length of uncompressed object
11->14          | Datime    | Date and time when object was written to file
15->16          | KeyLen    | Length of the key structure (in bytes)
17->18          | Cycle     | Cycle of key
19->22 [19->26] | SeekKey   | Pointer to record itself (consistency check)
23->26 [27->34] | SeekPdir  | Pointer to directory header
27->27 [35->35] | lname     | Number of bytes in the class name
28->.. [36->..] | ClassName | Object Class Name
..->..          | lname     | Number of bytes in the object name
..->..          | Name      | lName bytes with the name of the object
..->..          | lTitle    | Number of bytes in the object title
..->..          | Title     | Title of the object
----->          | DATA      | Data bytes associated to the object
 */
#[derive(Debug)]
pub struct TKeyHeader {
    pub n_bytes: u32,
    pub version: u16,
    pub obj_len: u32,
    pub datime: u32,
    pub key_len: u16,
    pub cycle: u16,
    pub seek_key: u64,
    pub seek_p_dir: u64,
    pub l_name: u8,
    pub class_name: String,
    pub name: String,
    pub title: String,
}

enum HeaderPtrWidth {
    Off32,
    Off64,
}

impl HeaderPtrWidth {
    fn new(f_unit: u8) -> Self {
        match f_unit {
            8 => HeaderPtrWidth::Off64,
            4 => HeaderPtrWidth::Off32,
            _ => panic!("Unexpected fUnits value: {}", f_unit),
        }
    }
}

impl TKeyHeader {
    pub fn new() -> Self {
        TKeyHeader {
            n_bytes: 0,
            version: 0,
            obj_len: 0,
            datime: 0,
            key_len: 0,
            cycle: 0,
            seek_key: 0,
            seek_p_dir: 0,
            l_name: 0,
            class_name: String::new(),
            name: String::new(),
            title: String::new(),
        }
    }

    pub fn read_tkey_at (reader: &mut BufReader<File>, offset: u64, f_unit: u8) -> io::Result<Self> {
        let header_ptr_width = HeaderPtrWidth::new(f_unit);
        reader.seek(SeekFrom::Start(offset))?;

        let read_hdr_ptr = |r: &mut BufReader<File>| -> io::Result<u64> {
            match header_ptr_width {
                HeaderPtrWidth::Off64 => r.read_u64::<BigEndian>(),
                HeaderPtrWidth::Off32 => Ok(r.read_u32::<BigEndian>()? as u64),
            }
        };
        let n_bytes = reader.read_u32::<BigEndian>()?;
        let version = reader.read_u16::<BigEndian>()?;
        let obj_len = reader.read_u32::<BigEndian>()?;
        let datime = reader.read_u32::<BigEndian>()?;
        let key_len = reader.read_u16::<BigEndian>()?;
        let cycle = reader.read_u16::<BigEndian>()?;
        let seek_key = read_hdr_ptr(reader)?;
        let seek_p_dir = read_hdr_ptr(reader)?;
        let l_name = Self::parse_lname(reader)?;
        Ok(TKeyHeader::new()) // Placeholder
    }
    fn parse_lname(reader: &mut BufReader<File>) -> io::Result<u8> {
        let mut lname_buf = [0u8; 1];
        reader.read_exact(&mut lname_buf)?;
        Ok(lname_buf[0])
    }
}   



