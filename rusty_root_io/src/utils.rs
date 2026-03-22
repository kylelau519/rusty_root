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
    Offset { offset: u32, class_name: String },
}

impl Default for ClassInfo {
    fn default() -> Self {
        ClassInfo::Offset {
            offset: 0,
            class_name: String::new(),
        }
    }
}

impl ClassInfo {
    pub fn read_class_info<R: Read + Seek>(reader: &mut R) -> io::Result<Self> {
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

            let current_pos = reader.stream_position()?;
            reader.seek(SeekFrom::Start(offset as u64 + 4))?; // 64 is key length, 4 is the tag we just read
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
            let class_name = String::from_utf8_lossy(&name_bytes).into_owned();

            Ok(ClassInfo::Offset { offset, class_name })
        }
    }

    pub fn get_class_name(&self) -> String {
        match self {
            ClassInfo::NewClass(name) => name.clone(),
            ClassInfo::Offset { class_name, .. } => class_name.clone(),
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

            let current_pos = reader.stream_position()?;
            reader.seek(SeekFrom::Start(offset as u64 + 4))?;

            let mut name_bytes = Vec::new();
            loop {
                let byte: u8 = reader.read_type(endian)?;
                if byte == 0 {
                    break;
                }
                name_bytes.push(byte);
            }
            reader.seek(SeekFrom::Start(current_pos))?;
            let class_name = String::from_utf8_lossy(&name_bytes).into_owned();

            Ok(ClassInfo::Offset { offset, class_name })
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
