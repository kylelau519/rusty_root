use crate::constant::K_BYTECOUNTMASK;
use crate::tnamed::TNamed;
use crate::tstring::TString;
use crate::utils::ClassInfo;
use binrw::io::{Read, Seek};
use binrw::{binread, BinRead, BinReaderExt, BinResult, Endian};

#[binread]
#[br(big)]
#[derive(Debug, Default)]
pub struct TStreamerElementBase {
    #[br(map = |x: u32| x & K_BYTECOUNTMASK)]
    pub byte_count: u32,
    pub version: u16,
    pub tnamed: TNamed,
    pub f_type: u32,
    pub f_size: u32,
    pub f_array_length: u32,
    pub f_array_dim: u32,
    pub f_max_index: [u32; 5],
    pub type_name: TString,
    // pub l_type_name: u8,
    // #[br(parse_with = binrw_read_string, args(l_type_name))]
    // pub type_name: String,
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
        name: TString,
        class_name: TString,
    },
    TStreamerObject,
    TStreamerObjectPointer,
    TStreamerLoop {
        count_version: u32,
        name: TString,
        class_name: TString,
    },
    TStreamerObjectAny,
    TStreamerSTL {
        stl_type: u32,
        c_type: u32,
    },
    TStreamerSTLString,
    Unknown(String),
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
                let name = reader.read_type(endian)?;
                let class_name = reader.read_type(endian)?;
                Ok(TStreamerType::TStreamerBasicPointer {
                    count_version,
                    name,
                    class_name,
                })
            }
            "TStreamerObject" => Ok(TStreamerType::TStreamerObject),
            "TStreamerObjectPointer" => Ok(TStreamerType::TStreamerObjectPointer),
            "TStreamerLoop" => {
                let count_version = reader.read_type(endian)?;
                let name = reader.read_type(endian)?;
                let class_name = reader.read_type(endian)?;
                Ok(TStreamerType::TStreamerLoop {
                    count_version,
                    name,
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
#[binread]
#[derive(Debug)]
pub struct TStreamerElement {
    #[br(map = |x: u32| x & K_BYTECOUNTMASK)]
    pub byte_count: u32,
    pub class_info: ClassInfo,
    #[br(map = |x: u32| x & K_BYTECOUNTMASK)]
    pub remaining_bytes: u32,
    pub version: u16,
    pub tstreamer_element_base: TStreamerElementBase,
    #[br(args(class_info.get_class_name()))]
    pub tstreamer_type: TStreamerType,
}
