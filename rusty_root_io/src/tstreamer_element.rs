use crate::tnamed::TNamed;
use crate::utils::ClassInfo;
use byteorder::ReadBytesExt;
use std::fs::File;
use std::io;
use std::io::BufReader;

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

// impl TStreamerElement {
//     pub fn read_tstreamer_element(reader: &mut BufReader<File>) -> io::Result<Self> {
//         let byte_count = reader.read_u32::<byteorder::BigEndian>()?;
//         let class_info = ClassInfo::read_class_info(reader)?;
//         let remaining_bytes = reader.read_u32::<byteorder::BigEndian>()?;
//         let version = reader.read_u16::<byteorder::BigEndian>()?;
//         let tstreamer_element_base = TStreamerElementBase::read_tstreamer_element_base(reader)?;
//         let tstreamer_type =
