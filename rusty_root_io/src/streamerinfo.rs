use crate::constant::{K_BYTECOUNTMASK, K_NEWCLASSTAG};
use crate::tbuf::TBuf;
use crate::tkey::TKey;
use crate::tlist::TList;
use std::io;

// https://root.cern/doc/v638/streamerinfo.html
#[derive(Debug, Default)]
pub struct StreamerInfo {
    pub streamer_info_header: TKey,
    pub tlist: TList,
    // pub tnamed: TNamed,
    pub tstreamerinfo: TStreamerInfo,
    // pub tobj_array: TObjArray,
}
#[derive(Debug, Default)]
pub struct TNamed {
    pub raw_byte_count: u32,
    pub version: u16,
    pub tobject: TObject,
    pub f_name: String,
    pub f_title: String,
}

impl TNamed {
    pub fn new_from_streamerinfo(tbuf: &mut TBuf) -> Result<Self, io::Error> {
        let raw_byte_count = tbuf.read_u32()?;
        let version = tbuf.read_u16()?;
        let tobject = TObject::new_from_streamerinfo(tbuf)?;
        let f_name_byte = tbuf.read_u8()?;
        let f_name = tbuf.read_string(f_name_byte as usize)?;
        let f_title_byte = tbuf.read_u8()?;
        let f_title = tbuf.read_string(f_title_byte as usize)?;
        Ok(TNamed {
            raw_byte_count,
            version,
            tobject,
            f_name,
            f_title,
        })
    }
}

#[derive(Debug, Default)]
pub struct TObject {
    pub version: u16,
    pub f_uniqueid: u32,
    pub f_bits: u32,
    pub pidf: Option<u16>,
}

impl TObject {
    pub fn new_from_streamerinfo(tbuf: &mut TBuf) -> Result<Self, io::Error> {
        let version = tbuf.read_u16()?;
        let f_uniqueid = tbuf.read_u32()?;
        let f_bits = tbuf.read_u32()?;
        let pidf = None;
        Ok(TObject {
            version,
            f_uniqueid,
            f_bits,
            pidf,
        })
    }
}

#[derive(Debug, Default)]
pub struct TStreamerInfo {
    pub raw_byte_count: u32,
    pub tag: ClassInfoTag,
    pub class_name: Option<String>,
    pub remain_bytes_after_header: u32,
    pub version: u16,
    pub tnamed: TNamed,
    pub f_checksum: u32,
    pub f_class_version: u32,
    pub tobj_array: TObjArray,
}

impl TStreamerInfo {
    pub fn new_from_streamerinfo(tbuf: &mut TBuf) -> Result<Self, io::Error> {
        let raw_byte_count = tbuf.read_u32()?;
        let byte_count = raw_byte_count & K_BYTECOUNTMASK;
        let tag = ClassInfoTag::from_tag(tbuf.read_u32()?);
        let mut class_name = None;
        match tag {
            ClassInfoTag::NewClass => {
                class_name = Some(tbuf.read_cstring(80)?);
                assert!(
                    class_name.as_deref() == Some("TStreamerInfo"),
                    "Expected TStreamerInfo, got {:?}",
                    class_name
                );
            }
            ClassInfoTag::ClIdx(_) => {
                todo!(); // not sure what to do here
            }
        }
        let remain_bytes_after_header = tbuf.read_u32()? | K_BYTECOUNTMASK;
        let version = tbuf.read_u16()?;
        let tnamed = TNamed::new_from_streamerinfo(tbuf)?;
        let f_checksum = tbuf.read_u32()?;
        let f_class_version = tbuf.read_u32()?;
        let tobj_array = TObjArray::new_from_streamerinfo(tbuf)?;
        Ok(TStreamerInfo {
            raw_byte_count,
            tag,
            class_name,
            remain_bytes_after_header,
            version,
            tnamed,
            f_checksum,
            f_class_version,
            tobj_array,
        })
    }
}

#[derive(Debug, Default)]
pub struct TObjArray {
    pub raw_byte_count: u32,
    pub tag: ClassInfoTag,
    pub class_name: Option<String>,
    pub version: u16,
    pub tobject: TObject,
    pub remain_byte_count: u32,
    pub f_name_byte: u8,
    pub f_name: String,
    pub n_objects: u32,
    pub f_lowerbound: u32,
}

impl TObjArray {
    pub fn new_from_streamerinfo(tbuf: &mut TBuf) -> Result<Self, io::Error> {
        let raw_byte_count = tbuf.read_u32()?;
        // let byte_count = raw_byte_count & K_BYTECOUNTMASK;
        let tag = ClassInfoTag::from_tag(tbuf.read_u32()?);
        let mut class_name = None;
        match tag {
            ClassInfoTag::NewClass => {
                class_name = Some(tbuf.read_cstring(80)?);
            }
            ClassInfoTag::ClIdx(_) => {
                todo!(); // not sure what to do here
            }
        }

        let remain_byte_count = tbuf.read_u32()?;
        let version = tbuf.read_u16()?;
        let tobject = TObject::new_from_streamerinfo(tbuf)?;
        let f_name_byte = tbuf.read_u8()?;
        let f_name = tbuf.read_string(f_name_byte as usize)?;
        let n_objects = tbuf.read_u32()?;
        let f_lowerbound = tbuf.read_u32()?;
        Ok(TObjArray {
            raw_byte_count,
            tag,
            class_name,
            version,
            tobject,
            remain_byte_count,
            f_name_byte,
            f_name,
            n_objects,
            f_lowerbound,
        })
    }
}

#[derive(Debug, Default)]
pub struct TStreamerElement {
    f_type: FType,
    f_size: u32,
    f_array_length: u32,
    f_array_dim: u32,
    f_max_index: [u32; 5],
    f_name_byte: u8,
    f_name: String,
}
impl TStreamerElement {
    pub fn new_from_streamerinfo(tbuf: &mut TBuf) -> Result<Self, io::Error> {
        let f_type = FType::try_from(tbuf.read_i32()?).map_err(|_| {
            io::Error::new(
                io::ErrorKind::InvalidData,
                "Invalid FType in TStreamerElement",
            )
        })?;
        let f_size = tbuf.read_u32()?;
        let f_array_length = tbuf.read_u32()?;
        let f_array_dim = tbuf.read_u32()?;
        let mut f_max_index = [0u32; 5];
        for i in 0..5 {
            f_max_index[i] = tbuf.read_u32()?;
        }
        let f_name_byte = tbuf.read_u8()?;
        let f_name = tbuf.read_string(f_name_byte as usize)?;
        Ok(TStreamerElement {
            f_type,
            f_size,
            f_array_length,
            f_array_dim,
            f_max_index,
            f_name_byte,
            f_name,
        })
    }
}

pub struct TStreamer {
    pub raw_byte_count: u32,
    pub tag: ClassInfoTag,
    pub class_name: Option<String>,
    pub byte_count_after_header: u32,
    pub version: u16,
    pub tstreamer_element: TStreamerElement,
    pub elements: Vec<TStreamerElement>,
}

impl StreamerInfo {}
