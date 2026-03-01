use crate::tnamed::TNamed;
use crate::tobjarray::TObjArray;
use crate::tstreamer_element::TStreamerElement;
use crate::utils::ClassInfo;

#[derive(Default)]
pub struct TStreamerInfo {
    pub byte_count: u32,
    pub class_info: ClassInfo,
    pub remaining_bytes: u32,
    pub version: u16,
    pub tnamed: TNamed,
    pub tobjarray: TObjArray<TStreamerElement>,
}
