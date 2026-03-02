use crate::constant::K_BYTECOUNTMASK;
use crate::tobject::TObject;
use crate::utils::ClassInfo;
use byteorder::ReadBytesExt;
use std::default::Default;
use std::io::{Read, Seek, SeekFrom};

#[derive(Debug)]
pub struct TObjArray<T> {
    pub byte_count: u32,
    pub class_info: ClassInfo,
    pub remaining_bytes: u32,
    pub version: u16,
    pub tobject: TObject,
    pub l_name: u8,
    pub name: String,
    pub n_objects: u32,
    pub f_lower_bound: i32,
    pub objects: Vec<T>,
}

impl<T> Default for TObjArray<T> {
    fn default() -> Self {
        Self {
            byte_count: 0,
            class_info: ClassInfo::default(),
            remaining_bytes: 0,
            version: 0,
            tobject: TObject::default(),
            l_name: 0,
            name: String::new(),
            n_objects: 0,
            f_lower_bound: 0,
            objects: Vec::new(),
        }
    }
}

impl<T> TObjArray<T> {
    pub fn read_tobjarray_at<R: Read + Seek>(reader: &mut R, offset: u64) -> std::io::Result<Self> {
        reader.seek(SeekFrom::Start(offset))?;
        let byte_count = reader.read_u32::<byteorder::BigEndian>()? & K_BYTECOUNTMASK;
        let class_info = ClassInfo::read_class_info(reader)?;
        let remaining_bytes = reader.read_u32::<byteorder::BigEndian>()? & K_BYTECOUNTMASK;
        let version = reader.read_u16::<byteorder::BigEndian>()?;
        let tobject = TObject::read_tobject(reader)?;
        let l_name = reader.read_u8()?;
        let name = crate::utils::read_string(reader, l_name as usize)?;
        let n_objects = reader.read_u32::<byteorder::BigEndian>()?;
        let f_lower_bound = reader.read_i32::<byteorder::BigEndian>()?;
        Ok(Self {
            byte_count,
            class_info,
            remaining_bytes,
            version,
            tobject,
            l_name,
            name,
            n_objects,
            f_lower_bound,
            objects: Vec::new(), // Placeholder, as we don't know the type T or how to read it yet
        })
    }
    // We will fill in the read function later, as we need to know how to read the objects of type T
}
