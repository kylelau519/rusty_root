use crate::constant::{K_BYTECOUNTMASK, K_NEWCLASSTAG};
use crate::tbuf::TBuf;
use crate::tkey::TKeyHeader;
use crate::tlist::TList;
use std::io;

// https://root.cern/doc/v638/streamerinfo.html
#[derive(Debug)]
pub enum ClassInfoTag {
    NewClass,
    ClIdx(u32),
}
impl ClassInfoTag {
    fn from_tag(tag: u32) -> Self {
        if (tag & K_NEWCLASSTAG) != 0 {
            ClassInfoTag::NewClass
        } else {
            ClassInfoTag::ClIdx(tag)
        }
    }
}
impl Default for ClassInfoTag {
    fn default() -> Self {
        ClassInfoTag::ClIdx(0)
    }
}

#[derive(Debug, Default)]
pub struct StreamerInfo {
    pub streamer_info_header: TKeyHeader,
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BuiltinDataType {
    Char = 1,
    Short = 2,
    Int = 3,
    Long = 4,
    Float = 5,
    Double = 8,
    UChar = 11,
    UShort = 12,
    UInt = 13,
    ULong = 14,
    ArrayDim = 6,
    BitMask = 15,
}
impl BuiltinDataType {
    fn from_code(c: i32) -> Option<Self> {
        use BuiltinDataType::*;
        Some(match c {
            1 => Char,
            2 => Short,
            3 => Int,
            4 => Long,
            5 => Float,
            8 => Double,
            11 => UChar,
            12 => UShort,
            13 => UInt,
            14 => ULong,
            15 => BitMask,
            6 => ArrayDim,
            _ => return None,
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FType {
    BaseClass,                       // 0
    Builtin(BuiltinDataType),        // 1.. etc.
    ArrayOfBuiltin(BuiltinDataType), // 20 + X
    PtrToBuiltin(BuiltinDataType),   // 40 + X
    TString,                         // 65
    TObject,                         // 66
    TNamed,                          // 67
    ObjDataMemberDerived,            // 61
    ObjDataMemberPlain,              // 62
    PtrObjNonNull,                   // 63
    PtrObjMaybeNull,                 // 64
    PtrArrayOfObjects,               // 501
    STLContainer,                    // 500 (includes std::string & containers)
}
impl Default for FType {
    fn default() -> Self {
        FType::BaseClass
    }
}

impl core::convert::TryFrom<i32> for FType {
    type Error = ();
    fn try_from(t: i32) -> Result<Self, Self::Error> {
        use FType::*;
        if t == 0 {
            return Ok(BaseClass);
        }
        if let Some(b) = BuiltinDataType::from_code(t) {
            return Ok(Builtin(b));
        }
        if (20..40).contains(&t) {
            if let Some(b) = BuiltinDataType::from_code(t - 20) {
                return Ok(ArrayOfBuiltin(b));
            }
        }
        if (40..60).contains(&t) {
            if let Some(b) = BuiltinDataType::from_code(t - 40) {
                return Ok(PtrToBuiltin(b));
            }
        }
        Ok(match t {
            61 => ObjDataMemberDerived,
            62 => ObjDataMemberPlain,
            63 => PtrObjNonNull,
            64 => PtrObjMaybeNull,
            65 => TString,
            66 => TObject,
            67 => TNamed,
            500 => STLContainer,
            501 => PtrArrayOfObjects,
            _ => return Err(()),
        })
    }
}

impl StreamerInfo {
    pub fn new(tkey: TKeyHeader) -> Result<Self, std::io::Error> {
        let data = match &tkey.decompressed_data {
            Some(d) => d,
            None => panic!("No decompressed data available"),
        };
        let mut cursor = TBuf::new(data);
        let tlist = TList::new_from_streamerinfo(&mut cursor)?;
        let tstreamerinfo = TStreamerInfo::new_from_streamerinfo(&mut cursor)?;
        // let tnamed = TNamed::new_from_streamerinfo(&mut cursor)?;
        // let f_checksum = cursor.read_u32()?;
        // let f_class_version = cursor.read_u32()?;
        // let tobj_array = TObjArray::new_from_streamerinfo(&mut cursor)?;

        Ok(StreamerInfo {
            streamer_info_header: tkey,
            tlist: tlist,
            // tnamed,
            tstreamerinfo,
            // tobj_array,
        })
    }
}
