use crate::constant::K_BYTECOUNTMASK;
use crate::tnamed::TNamed;
use crate::utils::{binrw_read_string, ClassInfo};
use binrw::{binread, BinRead, BinReaderExt, BinResult, Endian};
use byteorder::ReadBytesExt;
use std::io;
use std::io::{Read, Seek};

#[binread]
#[br(big)]
#[derive(Debug, Default)]
pub struct TStreamerElementBase {
    #[br(map = |x: u32| x & K_BYTECOUNTMASK)]
    byte_count: u32,
    version: u16,
    tnamed: TNamed,
    f_type: u32,
    f_size: u32,
    f_array_length: u32,
    f_array_dim: u32,
    f_max_index: [u32; 5],
    l_type_name: u8,
    #[br(parse_with = binrw_read_string, args(l_type_name))]
    type_name: String,
}

impl TStreamerElementBase {
    pub fn read_tstreamer_element_base<R: std::io::Read + std::io::Seek>(
        reader: &mut R,
    ) -> io::Result<Self> {
        let byte_count = reader.read_u32::<byteorder::BigEndian>()?;
        let version = reader.read_u16::<byteorder::BigEndian>()?;
        let tnamed = TNamed::read_tnamed(reader)?;
        let f_type = reader.read_u32::<byteorder::BigEndian>()?;
        let f_size = reader.read_u32::<byteorder::BigEndian>()?;
        let f_array_length = reader.read_u32::<byteorder::BigEndian>()?;
        let f_array_dim = reader.read_u32::<byteorder::BigEndian>()?;
        let mut f_max_index = [0u32; 5];
        for i in 0..5 {
            // Even though f_max_index is [u8; 5], in the file we should read 4 bytes 5 times
            let val = reader.read_u32::<byteorder::BigEndian>()?;
            f_max_index[i] = val;
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
    TStreamerBase {
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
    Unknown(String),
}

impl TStreamerType {
    pub fn read_streamer_type<R: std::io::Read + std::io::Seek>(
        reader: &mut R,
        class_name: &str,
    ) -> io::Result<Self> {
        match class_name {
            "TStreamerBase" => {
                let base_version = reader.read_u32::<byteorder::BigEndian>()?;
                Ok(TStreamerType::TStreamerBase { base_version })
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
            _ => {
                let pos = reader.stream_position().unwrap_or(0);
                eprintln!(
                    "⚠️ WARNING: Unknown TStreamerType '{}' at offset {:#X}",
                    class_name, pos
                );
                Ok(TStreamerType::Unknown(class_name.to_string()))
            }
        }
    }
}

impl BinRead for TStreamerType {
    // We define that this enum NEEDS a String argument to be parsed
    type Args<'a> = (String,);

    fn read_options<R: Read + Seek>(
        reader: &mut R,
        endian: Endian,
        args: Self::Args<'_>,
    ) -> BinResult<Self> {
        let (class_name,) = args;

        match class_name.as_str() {
            "TStreamerBase" => Ok(TStreamerType::TStreamerBase {
                base_version: reader.read_type(endian)?,
            }),
            "TStreamerBasicType" => Ok(TStreamerType::TStreamerBasicType),
            "TStreamerString" => Ok(TStreamerType::TStreamerString),
            "TStreamerBasicPointer" => {
                let count_version = reader.read_type(endian)?;
                let l_name = reader.read_type(endian)?;
                let name = binrw_read_string(reader, endian, l_name)?;
                let l_class_name = reader.read_type(endian)?;
                let class_name = binrw_read_string(reader, endian, l_class_name)?;
                Ok(TStreamerType::TStreamerBasicPointer {
                    count_version,
                    l_name: l_name.0,
                    name,
                    l_class_name: l_class_name.0,
                    class_name,
                })
            }
            "TStreamerObject" => Ok(TStreamerType::TStreamerObject),
            "TStreamerObjectPointer" => Ok(TStreamerType::TStreamerObjectPointer),
            "TStreamerLoop" => {
                let count_version = reader.read_type(endian)?;
                let l_name = reader.read_type(endian)?;
                let name = binrw_read_string(reader, endian, l_name)?;
                let l_class_name = reader.read_type(endian)?;
                let class_name = binrw_read_string(reader, endian, l_class_name)?;
                Ok(TStreamerType::TStreamerLoop {
                    count_version,
                    l_name: l_name.0,
                    name,
                    l_class_name: l_class_name.0,
                    class_name,
                })
            }
            "TStreamerObjectAny" => Ok(TStreamerType::TStreamerObjectAny),
            "TStreamerSTL" => Ok(TStreamerType::TStreamerSTL {
                stl_type: reader.read_type(endian)?,
                c_type: reader.read_type(endian)?,
            }),
            "TStreamerSTLString" => Ok(TStreamerType::TStreamerSTLString),
            _ => {
                let pos = reader.stream_position().unwrap_or(0);
                eprintln!(
                    "⚠️ WARNING: Unknown TStreamerType '{}' at offset {:#X}",
                    class_name, pos
                );
                Ok(TStreamerType::Unknown(class_name.to_string()))
            }
        }
    }
}
#[derive(Debug)]
pub struct TStreamerElement {
    pub byte_count: u32,
    pub class_info: ClassInfo,
    pub remaining_bytes: u32,
    pub version: u16,
    pub tstreamer_element_base: TStreamerElementBase,
    pub tstreamer_type: TStreamerType,
}

impl BinRead for TStreamerElement {
    type Args<'a> = ();

    fn read_options<R: Read + Seek>(
        reader: &mut R,
        endian: Endian,
        _args: Self::Args<'_>,
    ) -> BinResult<Self> {
        let byte_count = reader.read_type::<u32>(endian)? & K_BYTECOUNTMASK;
        let class_info = ClassInfo::read_be(reader)?;
        let remaining_bytes = reader.read_type::<u32>(endian)? & K_BYTECOUNTMASK;
        let version = reader.read_type::<u16>(endian)?;
        let tstreamer_element_base = TStreamerElementBase::read_be(reader)?;
        let streamer_type_string = class_info.get_class_name();
        let tstreamer_type = TStreamerType::read_streamer_type(reader, &streamer_type_string)?;
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
