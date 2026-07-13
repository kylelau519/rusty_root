use crate::core::constant::K_BYTECOUNTMASK;
use crate::core::utils::ClassInfo;
use crate::objects::tnamed::TNamed;
use crate::objects::tstring::TString;
use crate::streamer::registry::{ClassSchema, FieldKind};
use binrw::io::{Read, Seek};
use binrw::{binread, BinRead, BinReaderExt, BinResult, Endian};
use std::collections::HashMap;
use std::sync::Arc;

#[binread]
#[br(big)]
#[derive(Debug, Default)]
pub struct TStreamerElementBase {
    #[br(map = |x: u32| x & K_BYTECOUNTMASK)]
    pub byte_count: u32,
    pub version: u16,
    pub tnamed: TNamed,
    pub f_type: u32,
    pub f_size: u32,
    pub f_array_length: u32,
    pub f_array_dim: u32,
    pub f_max_index: [u32; 5],
    pub type_name: TString,
}

#[derive(Debug)]
pub enum TStreamerType {
    TStreamerBase {
        base_version: u32,
    },
    TStreamerBasicType,
    TStreamerString,
    TStreamerBasicPointer {
        count_version: u32,
        name: TString,
        class_name: TString,
    },
    TStreamerObject,
    TStreamerObjectPointer,
    TStreamerLoop {
        count_version: u32,
        name: TString,
        class_name: TString,
    },
    TStreamerObjectAny,
    TStreamerSTL {
        stl_type: u32,
        c_type: u32,
    },
    TStreamerSTLString,
    Unknown(String),
}

impl BinRead for TStreamerType {
    // We define that this enum NEEDS a String argument to be parsed
    type Args<'a> = (String,);

    fn read_options<R: Read + Seek>(
        reader: &mut R,
        endian: Endian,
        args: Self::Args<'_>,
    ) -> BinResult<Self> {
        let (class_name,) = args;

        match class_name.as_str() {
            "TStreamerBase" => Ok(TStreamerType::TStreamerBase {
                base_version: reader.read_type(endian)?,
            }),
            "TStreamerBasicType" => Ok(TStreamerType::TStreamerBasicType),
            "TStreamerString" => Ok(TStreamerType::TStreamerString),
            "TStreamerBasicPointer" => {
                let count_version = reader.read_type(endian)?;
                let name = reader.read_type(endian)?;
                let class_name = reader.read_type(endian)?;
                Ok(TStreamerType::TStreamerBasicPointer {
                    count_version,
                    name,
                    class_name,
                })
            }
            "TStreamerObject" => Ok(TStreamerType::TStreamerObject),
            "TStreamerObjectPointer" => Ok(TStreamerType::TStreamerObjectPointer),
            "TStreamerLoop" => {
                let count_version = reader.read_type(endian)?;
                let name = reader.read_type(endian)?;
                let class_name = reader.read_type(endian)?;
                Ok(TStreamerType::TStreamerLoop {
                    count_version,
                    name,
                    class_name,
                })
            }
            "TStreamerObjectAny" => Ok(TStreamerType::TStreamerObjectAny),
            "TStreamerSTL" => Ok(TStreamerType::TStreamerSTL {
                stl_type: reader.read_type(endian)?,
                c_type: reader.read_type(endian)?,
            }),
            "TStreamerSTLString" => Ok(TStreamerType::TStreamerSTLString),
            _ => {
                let pos = reader.stream_position().unwrap_or(0);
                eprintln!(
                    "⚠️ WARNING: Unknown TStreamerType '{}' at offset {:#X}",
                    class_name, pos
                );
                Ok(TStreamerType::Unknown(class_name.to_string()))
            }
        }
    }
}

#[binread]
#[derive(Debug)]
pub struct TStreamerElement {
    #[br(map = |x: u32| x & K_BYTECOUNTMASK)]
    pub byte_count: u32,
    pub class_info: ClassInfo,
    #[br(map = |x: u32| x & K_BYTECOUNTMASK)]
    pub remaining_bytes: u32,
    pub version: u16,
    pub tstreamer_element_base: TStreamerElementBase,
    #[br(args(class_info.get_class_name()))]
    pub tstreamer_type: TStreamerType,
}

impl TStreamerElement {
    pub fn to_field_kind(&self, registry: &HashMap<String, Arc<ClassSchema>>) -> FieldKind {
        let base = &self.tstreamer_element_base;
        match &self.tstreamer_type {
            TStreamerType::TStreamerBase { .. } => {
                let name = base.tnamed.name.as_str();
                // TObject::Streamer is one of ROOT's special no-byte-count classes
                // (see objects/tobject.rs) — it must never be read as a generic Base,
                // and it may not even have its own entry in this file's StreamerInfo.
                if name == "TObject" {
                    FieldKind::TObject
                } else {
                    match registry.get(name) {
                        Some(schema) => FieldKind::Base(schema.clone()),
                        None => FieldKind::Unknown,
                    }
                }
            }
            TStreamerType::TStreamerBasicType => {
                // f_type 61/62 (kObject/kAny) is an embedded object member
                // (TObject-derived / not, respectively) that ROOT serialized
                // via the "basic type" descriptor instead of
                // TStreamerObject(Any) — resolve it the same way those do.
                if base.f_type == 61 || base.f_type == 62 {
                    let class_name = base.type_name.trim_end_matches('\0');
                    match registry.get(class_name) {
                        Some(schema) => FieldKind::Object(schema.clone()),
                        None => FieldKind::Unknown,
                    }
                } else {
                    f_type_to_field_kind(base.f_type, base.f_array_length, "")
                }
            }
            TStreamerType::TStreamerString | TStreamerType::TStreamerSTLString => FieldKind::TString,
            TStreamerType::TStreamerBasicPointer { name, .. } => {
                f_type_to_field_kind(base.f_type, base.f_array_length, name)
            }
            TStreamerType::TStreamerObject | TStreamerType::TStreamerObjectAny => {
                let class_name = base.type_name.trim_end_matches('\0');
                match registry.get(class_name) {
                    Some(schema) => FieldKind::Object(schema.clone()),
                    None => FieldKind::Unknown,
                }
            }
            TStreamerType::TStreamerObjectPointer => {
                let class_name = base.type_name.trim_end_matches('*').trim_end_matches('\0');
                match registry.get(class_name) {
                    Some(schema) => FieldKind::ObjectPointer(schema.clone()),
                    None => FieldKind::Unknown,
                }
            }
            TStreamerType::TStreamerSTL { c_type, .. } => {
                FieldKind::StlVector(Box::new(f_type_to_field_kind(*c_type, 0, "")))
            }
            TStreamerType::TStreamerLoop { .. } | TStreamerType::Unknown(_) => FieldKind::Unknown,
        }
    }
}

// Maps ROOT's EReadWrite f_type codes to FieldKind.
// count_field is only used for variable-array cases (f_type 40-58).
fn f_type_to_field_kind(f_type: u32, f_array_length: u32, count_field: &str) -> FieldKind {
    let n = f_array_length as usize;
    let cf = || count_field.to_string();
    match f_type {
        // --- Primitives ---
        1  => FieldKind::I8,
        2  => FieldKind::I16,
        3  => FieldKind::I32,
        4  => FieldKind::I64,
        5  => FieldKind::F32,
        6  => FieldKind::I32,  // counter (array dimension variable)
        8  => FieldKind::F64,
        9  => FieldKind::F64,  // Double32
        11 => FieldKind::U8,
        12 => FieldKind::U16,
        13 => FieldKind::U32,
        14 => FieldKind::U64,
        15 => FieldKind::U32,  // bit mask
        16 => FieldKind::I64,  // Long64
        17 => FieldKind::U64,  // ULong64
        18 => FieldKind::Bool,
        65 => FieldKind::TString,
        66 => FieldKind::TObject,
        67 => FieldKind::TNamed,
        // --- Fixed arrays (f_type = 20 + primitive) ---
        21 => FieldKind::FixedArray { elem: Box::new(FieldKind::I8),   count: n },
        22 => FieldKind::FixedArray { elem: Box::new(FieldKind::I16),  count: n },
        23 => FieldKind::FixedArray { elem: Box::new(FieldKind::I32),  count: n },
        24 => FieldKind::FixedArray { elem: Box::new(FieldKind::I64),  count: n },
        25 => FieldKind::FixedArray { elem: Box::new(FieldKind::F32),  count: n },
        26 => FieldKind::FixedArray { elem: Box::new(FieldKind::I32),  count: n }, // counter[]
        28 => FieldKind::FixedArray { elem: Box::new(FieldKind::F64),  count: n },
        29 => FieldKind::FixedArray { elem: Box::new(FieldKind::F64),  count: n }, // Double32[]
        31 => FieldKind::FixedArray { elem: Box::new(FieldKind::U8),   count: n },
        32 => FieldKind::FixedArray { elem: Box::new(FieldKind::U16),  count: n },
        33 => FieldKind::FixedArray { elem: Box::new(FieldKind::U32),  count: n },
        34 => FieldKind::FixedArray { elem: Box::new(FieldKind::U64),  count: n },
        35 => FieldKind::FixedArray { elem: Box::new(FieldKind::U32),  count: n }, // bit mask[]
        36 => FieldKind::FixedArray { elem: Box::new(FieldKind::I64),  count: n }, // Long64[]
        37 => FieldKind::FixedArray { elem: Box::new(FieldKind::U64),  count: n }, // ULong64[]
        38 => FieldKind::FixedArray { elem: Box::new(FieldKind::Bool), count: n },
        // --- Variable arrays / pointers (f_type = 40 + primitive) ---
        41 => FieldKind::VariableArray { elem: Box::new(FieldKind::I8),   count_field: cf() },
        42 => FieldKind::VariableArray { elem: Box::new(FieldKind::I16),  count_field: cf() },
        43 => FieldKind::VariableArray { elem: Box::new(FieldKind::I32),  count_field: cf() },
        44 => FieldKind::VariableArray { elem: Box::new(FieldKind::I64),  count_field: cf() },
        45 => FieldKind::VariableArray { elem: Box::new(FieldKind::F32),  count_field: cf() },
        46 => FieldKind::VariableArray { elem: Box::new(FieldKind::I32),  count_field: cf() }, // counter*
        48 => FieldKind::VariableArray { elem: Box::new(FieldKind::F64),  count_field: cf() },
        49 => FieldKind::VariableArray { elem: Box::new(FieldKind::F64),  count_field: cf() }, // Double32*
        51 => FieldKind::VariableArray { elem: Box::new(FieldKind::U8),   count_field: cf() },
        52 => FieldKind::VariableArray { elem: Box::new(FieldKind::U16),  count_field: cf() },
        53 => FieldKind::VariableArray { elem: Box::new(FieldKind::U32),  count_field: cf() },
        54 => FieldKind::VariableArray { elem: Box::new(FieldKind::U64),  count_field: cf() },
        55 => FieldKind::VariableArray { elem: Box::new(FieldKind::U32),  count_field: cf() }, // bit mask*
        56 => FieldKind::VariableArray { elem: Box::new(FieldKind::I64),  count_field: cf() }, // Long64*
        57 => FieldKind::VariableArray { elem: Box::new(FieldKind::U64),  count_field: cf() }, // ULong64*
        58 => FieldKind::VariableArray { elem: Box::new(FieldKind::Bool), count_field: cf() },
        _  => FieldKind::Unknown,
    }
}
