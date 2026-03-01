use byteorder::{BigEndian, ReadBytesExt};
use std::fs::File;
use std::io;
use std::io::{BufReader, Read, Seek};

pub enum ReaderDynWidth {
    Off32,
    Off64,
}

impl ReaderDynWidth {
    pub fn from_tfile_version(version: u32) -> Self {
        if version >= 1_000_000 {
            ReaderDynWidth::Off64
        } else {
            ReaderDynWidth::Off32
        }
    }
    // This is used for TKey, which uses a different versioning scheme than
    pub fn from_tkey_version(version: u16) -> Self {
        if version >= 1000 {
            ReaderDynWidth::Off64
        } else {
            ReaderDynWidth::Off32
        }
    }

    pub fn from_unit(f_unit: u8) -> Self {
        match f_unit {
            8 => ReaderDynWidth::Off64,
            4 => ReaderDynWidth::Off32,
            _ => panic!("Unexpected fUnits value: {}", f_unit),
        }
    }

    pub fn read_ptr<R: Read>(&self, reader: &mut R) -> io::Result<u64> {
        match self {
            ReaderDynWidth::Off64 => reader.read_u64::<BigEndian>(),
            ReaderDynWidth::Off32 => Ok(reader.read_u32::<BigEndian>()? as u64),
        }
    }
}

// class info mainly for streamer info
#[derive(Debug, Clone, PartialEq)]
pub enum ClassInfo {
    NewClass(String),
    Offset(u32),
}

impl Default for ClassInfo {
    fn default() -> Self {
        ClassInfo::Offset(0)
    }
}

impl ClassInfo {
    pub fn read_class_info<R: Read>(reader: &mut R) -> io::Result<Self> {
        let tag = reader.read_u32::<BigEndian>()?;
        if tag == crate::constant::K_NEWCLASSTAG {
            let mut name = Vec::new();
            let mut byte = [0u8; 1];
            loop {
                reader.read_exact(&mut byte)?;
                if byte[0] == 0 {
                    break;
                }
                name.push(byte[0]);
            }
            let class_name = String::from_utf8_lossy(&name).into_owned();
            Ok(ClassInfo::NewClass(class_name))
        } else {
            let offset = if tag & crate::constant::K_NEW_CLASSBIT != 0 {
                (tag & !crate::constant::K_NEW_CLASSBIT) - crate::constant::K_MAP_OFFSET
            } else {
                tag
            };
            Ok(ClassInfo::Offset(offset))
        }
    }
}

pub fn decode_datime(datime: u32) -> String {
    let year = (datime >> 26) + 1995;
    let month = (datime >> 22) & 0xF;
    let day = (datime >> 17) & 0x1F;
    let hour = (datime >> 12) & 0x1F;
    let minute = (datime >> 6) & 0x3F;
    let second = datime & 0x3F;
    format!(
        "{:04}-{:02}-{:02} {:02}:{:02}:{:02}",
        year, month, day, hour, minute, second
    )
}

pub fn read_string<R: Read>(reader: &mut R, length: usize) -> io::Result<String> {
    let mut str_buf = vec![0u8; length];
    reader.read_exact(&mut str_buf)?;
    let s = String::from_utf8_lossy(&str_buf).to_string();
    Ok(s)
}

pub fn read_u1<R: Read>(reader: &mut R) -> io::Result<u8> {
    let mut buf = [0u8; 1];
    reader.read_exact(&mut buf)?;
    Ok(buf[0])
}
