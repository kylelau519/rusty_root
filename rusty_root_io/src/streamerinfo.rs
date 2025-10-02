use crate::tkey::{self, TKeyHeader};
use std::io::{self, Read, Seek};
use byteorder::{BigEndian, ReadBytesExt};
use crate::tbuf::TBuf;
use crate::constant::{K_BYTECOUNTMASK, K_HAS_BYTECOUNT};
use crate::tlist::TList;

#[derive(Debug, Default)]
pub struct StreamerInfo {
    pub streamer_info_header: TKeyHeader,
    pub tlist: TList,
    pub tnamed: TNamed,
    pub tstreamerinfo: TStreamerInfo,
    pub tobj_array: TObjArray,
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
    pub tag: u32,
    pub class_name: String,
    pub remain_bytes_after_header: u32,
    pub version: u16,
}

impl TStreamerInfo {
    pub fn new_from_streamerinfo(tbuf: &mut TBuf) -> Result<Self, io::Error> {
        let raw_byte_count = tbuf.read_u32()?;
        let byte_count = raw_byte_count & K_BYTECOUNTMASK;
        let tag = tbuf.read_u32()?;
        let class_name = tbuf.read_cstring(80)?;
        assert!(class_name == "TStreamerInfo", "Expected TStreamerInfo, got {}", class_name);
        let remain_bytes_after_header = tbuf.read_u32()? | K_BYTECOUNTMASK;
        let version = tbuf.read_u16()?;
        Ok(TStreamerInfo {
            raw_byte_count,
            tag,
            class_name,
            remain_bytes_after_header,
            version,
        })
    }
}

#[derive(Debug, Default)]
pub struct TObjArray {
    pub raw_byte_count: u32,
    pub tag: u32,
    pub class_name: String,
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
        let byte_count = raw_byte_count & K_BYTECOUNTMASK;
        let tag = tbuf.read_u32()?;
        let class_name = tbuf.read_cstring(80)?;
        assert!(class_name == "TObjArray", "Expected TObjArray, got {}", class_name);
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

impl StreamerInfo {
    pub fn new(tkey: TKeyHeader) -> Result<Self, std::io::Error> {
        let data = match &tkey.decompressed_data {
            Some(d) => d,
            None => panic!("No decompressed data available"),
        };
        let mut cursor = TBuf::new(data);
        let tlist = TList::new_from_streamerinfo(&mut cursor)?;
        let tstreamerinfo = TStreamerInfo::new_from_streamerinfo(&mut cursor)?;
        let tnamed = TNamed::new_from_streamerinfo(&mut cursor)?;
        let f_checksum = cursor.read_u32()?;
        let f_class_version = cursor.read_u32()?;
        let tobj_array = TObjArray::new_from_streamerinfo(&mut cursor)?;
        
        Ok(StreamerInfo {
            streamer_info_header: tkey,
            tlist: tlist,
            tnamed,
            tstreamerinfo,
            tobj_array,
        })
    }

}