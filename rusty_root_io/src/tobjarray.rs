use crate::tobject::TObject;
use std::default::Default;

#[derive(Debug)]
pub struct TObjArray<T> {
    pub byte_count: u32,
    pub class_info: u32, //fix later
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
            class_info: 0,
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
