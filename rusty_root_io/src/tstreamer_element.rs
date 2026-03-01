use crate::tnamed::TNamed;
use crate::utils::ClassInfo;
use byteorder::ReadBytesExt;
use std::fs::File;
use std::io;
use std::io::{BufReader, Read};

#[derive(Debug, Default)]
pub struct TStreamerElementBase {
    byte_count: u32,
    version: u16,
    tnamed: TNamed,
    f_type: u8,
    f_size: u32,
    f_array_length: u32,
    f_array_dim: u32,
    f_max_index: [u8; 5],
    l_type_name: u8,
    type_name: String,
}

impl TStreamerElementBase {
    pub fn read_tstreamer_element_base(reader: &mut BufReader<File>) -> io::Result<Self> {
        let byte_count = reader.read_u32::<byteorder::BigEndian>()?;
        let version = reader.read_u16::<byteorder::BigEndian>()?;
        let tnamed = TNamed::read_tnamed(reader)?;
        let f_type = reader.read_u8()?;
        let f_size = reader.read_u32::<byteorder::BigEndian>()?;
        let f_array_length = reader.read_u32::<byteorder::BigEndian>()?;
        let f_array_dim = reader.read_u32::<byteorder::BigEndian>()?;
        let mut f_max_index = [0u8; 5];
        for i in 0..5 {
            f_max_index[i] = reader.read_u8()?;
        }
        let l_type_name = reader.read_u8()?;
        let type_name = crate::utils::read_string(reader, l_type_name as usize)?;
        Ok(Self {
            byte_count,
            version,
            tnamed,
            f_type,
            f_size,
            f_array_length,
            f_array_dim,
            f_max_index,
            l_type_name,
            type_name,
        })
    }
}

#[derive(Debug)]
pub enum TStreamerType {
    TStreamerInfoBase {
        base_version: u32,
    },
    TStreamerBasicType,
    TStreamerString,
    TStreamerBasicPointer {
        count_version: u32,
        l_name: u8,
        name: String,
        l_class_name: u8,
        class_name: String,
    },
    TStreamerObject,
    TStreamerObjectPointer,
    TStreamerLoop {
        count_version: u32,
        l_name: u8,
        name: String,
        l_class_name: u8,
        class_name: String,
    },
    TStreamerObjectAny,
    TStreamerSTL {
        stl_type: u32,
        c_type: u32,
    },
    TStreamerSTLString,
}

pub struct TStreamerElement {
    pub byte_count: u32,
    pub class_info: ClassInfo,
    pub remaining_bytes: u32,
    pub version: u16,
    pub tstreamer_element_base: TStreamerElementBase,
    pub tstreamer_type: TStreamerType,
}

impl TStreamerType {
    pub fn read_streamer_type(reader: &mut BufReader<File>, class_name: &str) -> io::Result<Self> {
        match class_name {
            "TStreamerBase" => {
                let base_version = reader.read_u32::<byteorder::BigEndian>()?;
                Ok(TStreamerType::TStreamerInfoBase { base_version })
            }
            "TStreamerBasicType" => Ok(TStreamerType::TStreamerBasicType),
            "TStreamerString" => Ok(TStreamerType::TStreamerString),
            "TStreamerBasicPointer" => {
                let count_version = reader.read_u32::<byteorder::BigEndian>()?;
                let l_name = reader.read_u8()?;
                let name = crate::utils::read_string(reader, l_name as usize)?;
                let l_class_name = reader.read_u8()?;
                let class_name_str = crate::utils::read_string(reader, l_class_name as usize)?;
                Ok(TStreamerType::TStreamerBasicPointer {
                    count_version,
                    l_name,
                    name,
                    l_class_name,
                    class_name: class_name_str,
                })
            }
            "TStreamerObject" => Ok(TStreamerType::TStreamerObject),
            "TStreamerObjectPointer" => Ok(TStreamerType::TStreamerObjectPointer),
            "TStreamerLoop" => {
                let count_version = reader.read_u32::<byteorder::BigEndian>()?;
                let l_name = reader.read_u8()?;
                let name = crate::utils::read_string(reader, l_name as usize)?;
                let l_class_name = reader.read_u8()?;
                let class_name_str = crate::utils::read_string(reader, l_class_name as usize)?;
                Ok(TStreamerType::TStreamerLoop {
                    count_version,
                    l_name,
                    name,
                    l_class_name,
                    class_name: class_name_str,
                })
            }
            "TStreamerObjectAny" => Ok(TStreamerType::TStreamerObjectAny),
            "TStreamerSTL" => {
                let stl_type = reader.read_u32::<byteorder::BigEndian>()?;
                let c_type = reader.read_u32::<byteorder::BigEndian>()?;
                Ok(TStreamerType::TStreamerSTL { stl_type, c_type })
            }
            "TStreamerSTLString" => Ok(TStreamerType::TStreamerSTLString),
            _ => Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("Unknown TStreamerType class name: {}", class_name),
            )),
        }
    }
}

impl TStreamerElement {
    pub fn read_tstreamer_element(
        reader: &mut BufReader<File>,
        start_pos: u64,
    ) -> io::Result<Self> {
        use std::io::Seek;
        let byte_count = reader.read_u32::<byteorder::BigEndian>()?;
        let class_info = ClassInfo::read_class_info(reader)?;

        let class_name = match &class_info {
            ClassInfo::NewClass(name) => name.clone(),
            ClassInfo::Offset(offset) => {
                let current_pos = reader.stream_position()?;
                // The offset points to the tag (4 bytes), so we skip it to read the string
                reader.seek(std::io::SeekFrom::Start(start_pos + *offset as u64 + 4))?;
                let mut name_bytes = Vec::new();
                let mut byte = [0u8; 1];
                loop {
                    reader.read_exact(&mut byte)?;
                    if byte[0] == 0 {
                        break;
                    }
                    name_bytes.push(byte[0]);
                }
                // Seek back to continue reading
                reader.seek(std::io::SeekFrom::Start(current_pos))?;
                String::from_utf8_lossy(&name_bytes).into_owned()
            }
        };

        let remaining_bytes = reader.read_u32::<byteorder::BigEndian>()?;
        let version = reader.read_u16::<byteorder::BigEndian>()?;
        let tstreamer_element_base = TStreamerElementBase::read_tstreamer_element_base(reader)?;
        let tstreamer_type = TStreamerType::read_streamer_type(reader, &class_name)?;

        Ok(Self {
            byte_count,
            class_info,
            remaining_bytes,
            version,
            tstreamer_element_base,
            tstreamer_type,
        })
    }
}
