use crate::constant::K_BYTECOUNTMASK;
use crate::tobject::TObject;
use crate::utils::{binrw_read_string, ClassInfo};
use binrw::{binread, BinRead};
use byteorder::ReadBytesExt;
use std::default::Default;
use std::io::{Read, Seek, SeekFrom};

#[binread]
#[br(big)]
#[derive(Debug)]
pub struct TObjArray<T>
where
    // 1. T must own its data (no temporary references)
    T: BinRead + 'static,
    // 2. T must be readable with no arguments for any lifetime 'a
    for<'a> T: BinRead<Args<'a> = ()>,
{
    #[br(map = |x: u32| x & K_BYTECOUNTMASK)]
    pub byte_count: u32,
    pub class_info: ClassInfo,
    #[br(map = |x: u32| x & K_BYTECOUNTMASK)]
    pub remaining_bytes: u32,
    pub version: u16,
    pub tobject: TObject,
    pub l_name: u8,
    #[br(parse_with = binrw_read_string, args(l_name))]
    pub name: String,
    pub n_objects: u32,
    pub f_lower_bound: i32,
    #[br(count = n_objects)]
    pub objects: Vec<T>,
}

impl<T> Default for TObjArray<T>
where
    T: BinRead + 'static,
    for<'a> T: BinRead<Args<'a> = ()>,
{
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

impl<T> TObjArray<T>
where
    T: BinRead + 'static,
    for<'a> T: BinRead<Args<'a> = ()>,
{
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

        // At this point we really need a trait to read the objects of type T, we can only rely on binrw
        let mut objects = Vec::with_capacity(n_objects as usize);
        for _ in 0..n_objects {
            let obj = T::read_be(reader).expect("Failed to read object of type T");
            objects.push(obj);
        }
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
            objects,
        })
    }

    pub fn read_tobjarray<R: Read + Seek>(reader: &mut R) -> std::io::Result<Self> {
        let loc = reader.seek(SeekFrom::Current(0))?;
        Self::read_tobjarray_at(reader, loc)
    }
    // We will fill in the read function later, as we need to know how to read the objects of type T
}
