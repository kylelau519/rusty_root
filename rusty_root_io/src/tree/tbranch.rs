use crate::core::constant::{K_BYTECOUNTMASK, K_HAS_BYTECOUNT, K_NEWCLASSTAG};
use crate::core::utils::ClassInfo;
use crate::objects::tnamed::TNamed;
use crate::objects::tobjarray::TObjArray;
use crate::objects::tstring::TString;
use crate::tree::tleaf::AnyLeaf;
use binrw::{BinRead, BinReaderExt, BinResult, Endian};
use binrw::io::{Read, Seek, SeekFrom};

/// Extra fields present only on TBranchElement (v7 schema).
#[derive(Debug)]
pub struct TBranchElementData {
    pub f_class_name: TString,
    pub f_parent_name: TString,
    pub f_clones_name: TString,
    pub f_class_version: i32,
    pub f_id: i32,
    pub f_type: i32,
    pub f_streamer_type: i32,
    pub f_maximum: i32,
}

/// TBranch (v7) and TBranchElement (v7).
/// Both start with the same outer envelope; TBranchElement adds an extra
/// ReadVersion pair before the TBranch base, then extra fields after.
#[derive(Debug)]
pub struct TBranch {
    pub byte_count: u32,
    pub class_info: ClassInfo,
    pub remaining: u32,
    pub version: u16,
    pub tnamed: TNamed,
    pub f_compress: i32,
    pub f_basket_size: i32,
    pub f_entry_offset_len: i32,
    pub f_write_basket: i32,
    pub f_entry_number: i32,
    pub f_offset: i32,
    pub f_max_baskets: i32,
    pub f_split_level: i32,
    pub f_entries: f64,
    pub f_tot_bytes: f64,
    pub f_zip_bytes: f64,
    pub f_branches: TObjArray<TBranch>,
    pub f_leaves: TObjArray<AnyLeaf>,
    // fBaskets skipped via byte_count — always null in a stored TTree
    pub f_basket_bytes: Vec<i32>,
    pub f_basket_entry: Vec<i32>,
    pub f_basket_seek: Vec<i32>, // i32 / Seek_t in v7; wider in newer versions
    pub f_file_name: TString,
    pub branch_element: Option<TBranchElementData>,
}

impl BinRead for TBranch {
    type Args<'a> = ();

    fn read_options<R: Read + Seek>(reader: &mut R, endian: Endian, _: ()) -> BinResult<Self> {
        let byte_count = reader.read_type::<u32>(endian)? & K_BYTECOUNTMASK;
        let class_info = ClassInfo::read_options(reader, endian, ())?;
        let class_name = class_info.get_class_name();

        // TBranchElement::Streamer calls ReadVersion for itself, then calls
        // TBranch::Streamer which calls its own ReadVersion. Plain TBranch has
        // only its own ReadVersion.
        let (remaining, version) = if class_name == "TBranchElement" {
            let _elem_remaining = reader.read_type::<u32>(endian)? & K_BYTECOUNTMASK;
            let _elem_version: u16 = reader.read_type(endian)?;
            let rem = reader.read_type::<u32>(endian)? & K_BYTECOUNTMASK;
            let ver: u16 = reader.read_type(endian)?;
            (rem, ver)
        } else {
            let rem = reader.read_type::<u32>(endian)? & K_BYTECOUNTMASK;
            let ver: u16 = reader.read_type(endian)?;
            (rem, ver)
        };

        let tnamed = TNamed::read_options(reader, endian, ())?;
        let f_compress: i32 = reader.read_type(endian)?;
        let f_basket_size: i32 = reader.read_type(endian)?;
        let f_entry_offset_len: i32 = reader.read_type(endian)?;
        let f_write_basket: i32 = reader.read_type(endian)?;
        let f_entry_number: i32 = reader.read_type(endian)?;
        let f_offset: i32 = reader.read_type(endian)?;
        let f_max_baskets: i32 = reader.read_type(endian)?;
        let f_split_level: i32 = reader.read_type(endian)?;
        let f_entries: f64 = reader.read_type(endian)?;
        let f_tot_bytes: f64 = reader.read_type(endian)?;
        let f_zip_bytes: f64 = reader.read_type(endian)?;

        let f_branches = TObjArray::<TBranch>::read_options(reader, endian, ())?;
        let f_leaves = TObjArray::<AnyLeaf>::read_options(reader, endian, ())?;

        // Skip fBaskets TObjArray using its byte_count prefix.
        let raw: u32 = reader.read_type(endian)?;
        if raw & K_HAS_BYTECOUNT != 0 {
            reader.seek(SeekFrom::Current((raw & K_BYTECOUNTMASK) as i64))?;
        }

        // fBasketBytes, fBasketEntry, fBasketSeek: TStreamerBasicPointer format
        // = i32 count followed by count × element.
        let f_basket_bytes = read_i32_array(reader, endian)?;
        let f_basket_entry = read_i32_array(reader, endian)?;
        let f_basket_seek = read_i32_array(reader, endian)?;

        let f_file_name = TString::read_options(reader, endian, ())?;

        // TBranchElement extra fields (only present when class is TBranchElement).
        let branch_element = if class_name == "TBranchElement" {
            let f_class_name = TString::read_options(reader, endian, ())?;
            let f_parent_name = TString::read_options(reader, endian, ())?;
            let f_clones_name = TString::read_options(reader, endian, ())?;
            let f_class_version: i32 = reader.read_type(endian)?;
            let f_id: i32 = reader.read_type(endian)?;
            let f_type: i32 = reader.read_type(endian)?;
            let f_streamer_type: i32 = reader.read_type(endian)?;
            let f_maximum: i32 = reader.read_type(endian)?;
            // fBranchCount / fBranchCount2: TBranchElement* — consume pointer tag(s)
            consume_ptr(reader, endian)?;
            consume_ptr(reader, endian)?;
            Some(TBranchElementData {
                f_class_name, f_parent_name, f_clones_name,
                f_class_version, f_id, f_type, f_streamer_type, f_maximum,
            })
        } else {
            None
        };

        Ok(TBranch {
            byte_count, class_info, remaining, version,
            tnamed, f_compress, f_basket_size, f_entry_offset_len,
            f_write_basket, f_entry_number, f_offset, f_max_baskets,
            f_split_level, f_entries, f_tot_bytes, f_zip_bytes,
            f_branches, f_leaves,
            f_basket_bytes, f_basket_entry, f_basket_seek,
            f_file_name, branch_element,
        })
    }
}

/// Read a TStreamerBasicPointer counted array: i32 n, then n × i32 elements.
fn read_i32_array<R: Read + Seek>(reader: &mut R, endian: Endian) -> BinResult<Vec<i32>> {
    let n: i32 = reader.read_type(endian)?;
    let n = n.max(0) as usize;
    let mut v = Vec::with_capacity(n);
    for _ in 0..n {
        v.push(reader.read_type(endian)?);
    }
    Ok(v)
}

/// Consume a nullable object pointer tag.
/// Null = 0x00000000 (4 bytes).
/// Offset reference = 4 bytes (K_NEW_CLASSBIT set).
/// New class = K_NEWCLASSTAG + null-terminated class name (rare here).
fn consume_ptr<R: Read + Seek>(reader: &mut R, endian: Endian) -> BinResult<()> {
    let tag: u32 = reader.read_type(endian)?;
    if tag == K_NEWCLASSTAG {
        loop {
            let b: u8 = reader.read_type(endian)?;
            if b == 0 { break; }
        }
    }
    Ok(())
}
