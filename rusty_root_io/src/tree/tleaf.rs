use crate::core::constant::K_BYTECOUNTMASK;
use crate::core::utils::ClassInfo;
use crate::objects::tnamed::TNamed;
use binrw::{BinRead, BinReaderExt, BinResult, Endian};
use binrw::io::{Read, Seek};

/// Common fields for all TLeaf subtypes (TLeaf v2 schema).
#[binrw::binread]
#[br(big)]
#[derive(Debug, Default)]
pub struct TLeafBase {
    #[br(map = |x: u32| x & K_BYTECOUNTMASK)]
    pub byte_count: u32,
    pub version: u16,
    pub tnamed: TNamed,
    pub f_len: i32,
    pub f_len_type: i32,
    pub f_offset: i32,
    #[br(map = |x: u8| x != 0)]
    pub f_is_range: bool,
    #[br(map = |x: u8| x != 0)]
    pub f_is_unsigned: bool,
    // fLeafCount: TLeaf* — null pointer in simple branches, consume the tag
    #[br(temp)]
    _leaf_count_tag: u32,
}

/// Generic typed leaf: TLeafD, TLeafF, TLeafI, TLeafL, TLeafB.
/// T is the element primitive type (f64, f32, i32, i64, i8).
#[derive(Debug)]
pub struct TLeaf<T> {
    pub byte_count: u32,
    pub class_info: ClassInfo,
    pub remaining: u32,
    pub version: u16,
    pub base: TLeafBase,
    pub f_maximum: T,
}

/// TLeafElement — for TBranchElement branches (complex/split objects).
#[derive(Debug)]
pub struct TLeafElement {
    pub byte_count: u32,
    pub class_info: ClassInfo,
    pub remaining: u32,
    pub version: u16,
    pub base: TLeafBase,
    pub f_id: i32,
    pub f_type: i32,
}

/// Dispatching enum over all leaf subtypes.
/// Used as T in TObjArray<AnyLeaf> inside TBranch.
#[derive(Debug)]
pub enum AnyLeaf {
    D(TLeaf<f64>),
    F(TLeaf<f32>),
    I(TLeaf<i32>),
    L(TLeaf<i64>),
    B(TLeaf<i8>),
    Element(TLeafElement),
}

impl BinRead for AnyLeaf {
    type Args<'a> = ();

    fn read_options<R: Read + Seek>(reader: &mut R, endian: Endian, _: ()) -> BinResult<Self> {
        let byte_count = reader.read_type::<u32>(endian)? & K_BYTECOUNTMASK;
        let class_info = ClassInfo::read_options(reader, endian, ())?;
        let remaining = reader.read_type::<u32>(endian)? & K_BYTECOUNTMASK;
        let version: u16 = reader.read_type(endian)?;
        let base = TLeafBase::read_options(reader, endian, ())?;

        let class_name = class_info.get_class_name();
        match class_name.as_str() {
            "TLeafD" => Ok(AnyLeaf::D(TLeaf {
                byte_count, class_info, remaining, version, base,
                f_maximum: reader.read_type(endian)?,
            })),
            "TLeafF" => Ok(AnyLeaf::F(TLeaf {
                byte_count, class_info, remaining, version, base,
                f_maximum: reader.read_type(endian)?,
            })),
            "TLeafI" => Ok(AnyLeaf::I(TLeaf {
                byte_count, class_info, remaining, version, base,
                f_maximum: reader.read_type(endian)?,
            })),
            "TLeafL" => Ok(AnyLeaf::L(TLeaf {
                byte_count, class_info, remaining, version, base,
                f_maximum: reader.read_type(endian)?,
            })),
            "TLeafB" => Ok(AnyLeaf::B(TLeaf {
                byte_count, class_info, remaining, version, base,
                f_maximum: reader.read_type(endian)?,
            })),
            "TLeafElement" => Ok(AnyLeaf::Element(TLeafElement {
                byte_count, class_info, remaining, version, base,
                f_id: reader.read_type(endian)?,
                f_type: reader.read_type(endian)?,
            })),
            other => Err(binrw::Error::Custom {
                pos: reader.stream_position().unwrap_or(0),
                err: Box::new(format!("unknown TLeaf class: {other}")),
            }),
        }
    }
}
