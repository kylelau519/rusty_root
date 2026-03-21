use crate::constant::K_BYTECOUNTMASK;
use crate::tobject::TObject;
use crate::utils::{binrw_read_string, ClassInfo};
use binrw::{binread, BinRead};
use std::default::Default;

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
