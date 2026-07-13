use crate::core::constant::K_BYTECOUNTMASK;
use crate::objects::tnamed::TNamed;
use crate::objects::tobjarray::TObjArray;
use crate::tree::tarray::TArray;
use crate::tree::tbranch::TBranch;
use crate::tree::tleaf::AnyLeaf;

// TAttLine / TAttFill / TAttMarker are base classes of TTree.
// Each is streamed with its own byte_count + version header (TStreamerBase, type=0).

#[binrw::binread]
#[br(big)]
#[derive(Debug, Default)]
pub struct TAttLine {
    #[br(map = |x: u32| x & K_BYTECOUNTMASK)]
    pub byte_count: u32,
    pub version: u16,
    pub f_line_color: i16,
    pub f_line_style: i16,
    pub f_line_width: i16,
}

#[binrw::binread]
#[br(big)]
#[derive(Debug, Default)]
pub struct TAttFill {
    #[br(map = |x: u32| x & K_BYTECOUNTMASK)]
    pub byte_count: u32,
    pub version: u16,
    pub f_fill_color: i16,
    pub f_fill_style: i16,
}

#[binrw::binread]
#[br(big)]
#[derive(Debug, Default)]
pub struct TAttMarker {
    #[br(map = |x: u32| x & K_BYTECOUNTMASK)]
    pub byte_count: u32,
    pub version: u16,
    pub f_marker_color: i16,
    pub f_marker_style: i16,
    pub f_marker_size: f32,
}

/// TTree (v6 schema).
/// Stored as the root object of a TKey payload — no outer ClassInfo,
/// starts directly with ReadVersion output (byte_count + version).
#[binrw::binread]
#[br(big)]
#[derive(Debug)]
pub struct TTree {
    #[br(map = |x: u32| x & K_BYTECOUNTMASK)]
    pub byte_count: u32,
    pub version: u16,
    pub tnamed: TNamed,
    pub t_att_line: TAttLine,
    pub t_att_fill: TAttFill,
    pub t_att_marker: TAttMarker,
    pub f_entries: f64,
    pub f_tot_bytes: f64,
    pub f_zip_bytes: f64,
    pub f_saved_bytes: f64,
    pub f_timer_interval: i32,
    pub f_scan_field: i32,
    pub f_update: i32,
    pub f_max_entry_loop: i32,
    pub f_max_virtual_size: i32,
    pub f_auto_save: i32,
    pub f_estimate: i32,
    pub f_branches: TObjArray<TBranch>,
    pub f_leaves: TObjArray<AnyLeaf>,
    pub f_index_values: TArray<f64>,
    pub f_index: TArray<i32>,
    // fFriends: TList* — null pointer in most physics files, consume the tag
    #[br(temp)]
    _f_friends_null: u32,
}

impl TTree {
    pub fn n_entries(&self) -> i64 {
        self.f_entries as i64
    }

    pub fn branches(&self) -> impl Iterator<Item = &TBranch> {
        self.f_branches.iter()
    }

    pub fn branch(&self, name: &str) -> Option<&TBranch> {
        self.f_branches.iter().find(|b| b.tnamed.name.as_str() == name)
    }
}
