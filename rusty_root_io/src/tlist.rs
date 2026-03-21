use crate::tobject::TObject;
use crate::utils::binrw_read_string;
use binrw::BinRead;
use std::ops::Deref;

#[binrw::binread]
#[derive(Default, Debug)]
pub struct TList<T>
where
    // 1. T must own its data (no temporary references)
    T: BinRead + 'static,
    // 2. T must be readable with no arguments for any lifetime 'a
    for<'a> T: BinRead<Args<'a> = ()>,
{
    #[br(map = |x: u32| x & crate::constant::K_BYTECOUNTMASK)]
    pub byte_count: u32,
    pub version: u16,
    pub tobject: TObject,
    pub f_name_byte: u8,
    #[br(parse_with = binrw_read_string, args(f_name_byte))]
    pub f_name: String,
    pub n_objects: u32,
    #[br(count = n_objects)]
    pub objects: Vec<TListElement<T>>,
}

// TListElement is needed because in TList every object is read and followed a 'l_option" and a "option_string", not written in the link and is hidden...
#[binrw::binread]
#[derive(Default, Debug)]
pub struct TListElement<T>
where
    T: BinRead + 'static,
    for<'a> T: BinRead<Args<'a> = ()>,
{
    pub object: T,
    pub option_len: u8,
    #[br(parse_with = binrw_read_string, args(option_len))]
    pub option: String,
}

impl<T> Deref for TListElement<T>
where
    T: BinRead + 'static,
    for<'a> T: BinRead<Args<'a> = ()>,
{
    type Target = T;
    fn deref(&self) -> &Self::Target {
        &self.object
    }
}
