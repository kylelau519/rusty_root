use binrw::io::{Read, Seek, SeekFrom};
use binrw::{BinRead, BinReaderExt, BinResult, Endian};
use std::collections::HashMap;
use std::{fmt, sync::Arc};

use crate::core::constant::{
    K_BYTECOUNTMASK, K_HAS_BYTECOUNT, K_NEWCLASSTAG, K_NEW_CLASSBIT, K_NULLTAG,
};
use crate::objects::tnamed::TNamed;
use crate::objects::tobject::TObject;
use crate::objects::tstring::TString;

pub trait ReadSeek: Read + Seek {}
impl<T: Read + Seek> ReadSeek for T {}

#[derive(Clone)]
pub struct ClassSchema {
    pub name: String,
    pub version: u16,
    pub checksum: u32,
    pub fields: Vec<FieldSchema>, // ordered, base class fields first
    pub plan: Vec<CompiledReader>,
    pub plan_completed: bool, // whether the plan is completed, if not, it may be missing some base class or object class
}

impl fmt::Debug for ClassSchema {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ClassSchema")
            .field("name", &self.name)
            .field("version", &self.version)
            .field("checksum", &self.checksum)
            .field("fields", &self.fields)
            .field("plan_completed", &self.plan_completed)
            .finish()
    }
}

#[derive(Clone, Debug)]
pub struct FieldSchema {
    pub name: String,
    pub kind: FieldKind,
}

#[derive(Clone, Debug)]
// FieldKind gives enough info to build a reader, If the type is a pointer to an int, it should help building a reader to read that pointer (8 byte), instead of actually getting the  data
// note2: To resolve type 0 and type 62 61, there is a completed registry {name: vec<read instruction>}
// when it see a base, check the completed registry, if not there, continue(maybe build the missing first but potentially get into inf loop), else finish the builder and pop it out, until the registry is exhausted.
// DAG can resolve this but too troublesome to implement
pub enum FieldKind {
    // Primitives (f_type 1-5, 8-9, 11-18)
    Bool,
    I8,
    I16,
    I32,
    I64,
    U8,
    U16,
    U32,
    U64,
    F32,
    F64,
    TString, // Built-in objects (f_type 65-67)
    TObject,
    TNamed,
    // Fixed-size array of primitive, count from f_array_length (f_type 20+x)
    FixedArray { elem: Box<FieldKind>, count: usize },
    PrimativePointer { elem: Box<FieldKind> },
    // Object types
    // There should be a registry of completed reader, contain name: vec<byte reading order>
    Base { elem: String },
    Object { elem: String },
    // pointer generically read the byte, it may not be convenient to access the info but the builder is correct
    // Read the info later through the name.
    ObjectPointer { elem: String },
    StlVector { count: usize }, //assume it is vector<string>
    Unknown,
}
// Arc, not Box: a Base/Object field reuses the nested schema's already-built
// plan wholesale (see register_tstreamerinfo), which means cloning entries out
// of it. Arc::clone is a refcount bump regardless of the closure inside; Box
// has no such Clone impl since it can't generically duplicate the boxed value.
//

pub type CompiledReader = Arc<dyn for<'a> Fn(&'a mut dyn ReadSeek) + Send + Sync>;

pub fn build_field_reader(
    field_kind: &FieldKind,
    registry: &HashMap<String, Arc<ClassSchema>>,
) -> CompiledReader {
    match field_kind {
        FieldKind::Bool => Arc::new(|mut reader| {
            let _value = reader.read_type::<u8>(Endian::Big).unwrap();
            let _bool = _value != 0;
        }),
        FieldKind::I8 => Arc::new(|mut reader| {
            let _value = reader.read_type::<i8>(Endian::Big).unwrap();
        }),
        FieldKind::I16 => Arc::new(|mut reader| {
            let _value = reader.read_type::<i16>(Endian::Big).unwrap();
        }),
        FieldKind::I32 => Arc::new(|mut reader| {
            let _value = reader.read_type::<i32>(Endian::Big).unwrap();
        }),
        FieldKind::I64 => Arc::new(|mut reader| {
            let _value = reader.read_type::<i64>(Endian::Big).unwrap();
        }),
        FieldKind::U8 => Arc::new(|mut reader| {
            let _value = reader.read_type::<u8>(Endian::Big).unwrap();
        }),
        FieldKind::U16 => Arc::new(|mut reader| {
            let _value = reader.read_type::<u16>(Endian::Big).unwrap();
        }),
        FieldKind::U32 => Arc::new(|mut reader| {
            let _value = reader.read_type::<u32>(Endian::Big).unwrap();
        }),
        FieldKind::U64 => Arc::new(|mut reader| {
            let _value = reader.read_type::<u64>(Endian::Big).unwrap();
        }),
        FieldKind::F32 => Arc::new(|mut reader| {
            let _value = reader.read_type::<f32>(Endian::Big).unwrap();
        }),
        FieldKind::F64 => Arc::new(|mut reader| {
            let _value = reader.read_type::<f64>(Endian::Big).unwrap();
        }),
        FieldKind::TString => Arc::new(|mut reader| {
            let _value = TString::read_options(&mut reader, Endian::Big, ()).unwrap();
        }),
        FieldKind::TObject => Arc::new(|mut reader| {
            let _value = TObject::read_options(&mut reader, Endian::Big, ()).unwrap();
        }),
        FieldKind::TNamed => Arc::new(|mut reader| {
            let _value = TNamed::read_options(&mut reader, Endian::Big, ()).unwrap();
        }),
        FieldKind::FixedArray { elem, count } => {
            let count = *count;
            let elem_reader = build_field_reader(elem, registry); //This function outlive the frame, so need to move
            Arc::new(move |mut reader| {
                for _ in 0..count {
                    elem_reader(&mut *reader); // reborrow each iteration — elem_reader takes reader by value
                }
            })
        }
        FieldKind::Base { elem } => {
            let elem_schema = registry
                .get(elem)
                .expect("Base class not found in registry");
            let base_plan = elem_schema.plan.clone();
            Arc::new(move |mut reader| {
                for step in base_plan.iter() {
                    step(&mut *reader);
                }
            })
        }
        FieldKind::Object { elem } => todo!("Object reader not implemented yet"),
        FieldKind::ObjectPointer { elem } => Arc::new(|mut reader| {
            let _value = reader.read_type::<u64>(Endian::Big).unwrap(); //think about how to get the type later...from schema?
        }),
        FieldKind::StlVector { count } => todo!("StlVector reader not implemented yet"),
        FieldKind::PrimativePointer { elem } => todo!("StlVector reader not implemented yet"),
        FieldKind::Unknown => todo!("Unknown field reader not implemented yet"),
    }
}
