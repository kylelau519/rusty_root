use crate::constant::{K_MAP_OFFSET, K_NEWCLASSTAG, K_NEW_CLASSBIT};
use binrw::{BinRead, BinReaderExt, BinResult, Endian};
use byteorder::{BigEndian, ReadBytesExt};
use std::io;
use std::io::{Read, Seek, SeekFrom};

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
            let offset = if tag & K_NEW_CLASSBIT != 0 {
                (tag & !K_NEW_CLASSBIT) - K_MAP_OFFSET
            } else {
                tag
            };
            Ok(ClassInfo::Offset(offset))
        }
    }

    pub fn get_class_name<R: Read + Seek>(&self, reader: &mut R) -> io::Result<String> {
        match self {
            ClassInfo::NewClass(name) => Ok(name.clone()),
            ClassInfo::Offset(offset) => {
                let current_pos = reader.seek(SeekFrom::Current(0))?;
                reader.seek(SeekFrom::Start(*offset as u64))?;
                let mut name_bytes = Vec::new();
                let mut byte = [0u8; 1];
                loop {
                    reader.read_exact(&mut byte)?;
                    if byte[0] == 0 {
                        break;
                    }
                    name_bytes.push(byte[0]);
                }
                reader.seek(SeekFrom::Start(current_pos))?;
                Ok(String::from_utf8_lossy(&name_bytes).into_owned())
            }
        }
    }
}

impl BinRead for ClassInfo {
    type Args<'a> = ();

    fn read_options<R: Read + Seek>(
        reader: &mut R,
        endian: Endian,
        _args: Self::Args<'_>,
    ) -> BinResult<Self> {
        // 1. Read the 32-bit tag
        let tag: u32 = reader.read_type(endian)?;

        if tag == K_NEWCLASSTAG {
            // 2. Handle NewClass: Read null-terminated string
            let mut name_bytes = Vec::new();
            loop {
                let byte: u8 = reader.read_type(endian)?;
                if byte == 0 {
                    break;
                }
                name_bytes.push(byte);
            }
            let name = String::from_utf8_lossy(&name_bytes).into_owned();
            Ok(ClassInfo::NewClass(name))
        } else {
            // 3. Handle Offset: Perform bitwise logic
            let offset = if (tag & K_NEW_CLASSBIT) != 0 {
                (tag & !K_NEW_CLASSBIT) - K_MAP_OFFSET
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

pub fn debug_in_ascii(bytes: &[u8]) -> String {
    bytes
        .iter()
        .map(|&b| {
            if b.is_ascii_graphic() || b == b' ' {
                (b as char).to_string()
            } else {
                format!("[{:02x}]", b)
            }
        })
        .collect::<String>()
}

pub fn binrw_read_string<R: Read + Seek>(
    reader: &mut R,
    endian: Endian,
    args: (u8,), // This receives the 'l_name' we just read
) -> BinResult<String> {
    let (l_name,) = args;

    let length = if l_name == 255 {
        // Overflow case: read the next 4 bytes as the true length
        reader.read_type::<u32>(endian)?
    } else {
        l_name as u32
    };

    if length == 0 {
        return Ok(String::new());
    }

    let mut buf = vec![0u8; length as usize];
    reader.read_exact(&mut buf)?;
    Ok(String::from_utf8_lossy(&buf).into_owned())
}
