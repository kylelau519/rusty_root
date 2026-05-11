use std::sync::Arc;

#[derive(Debug)]
pub struct ClassSchema {
    pub name: String,
    pub version: u16,
    pub checksum: u32,
    pub fields: Vec<FieldSchema>, // ordered, base class fields first
}

#[derive(Debug)]
pub struct FieldSchema {
    pub name: String,
    pub kind: FieldKind,
}

#[derive(Debug)]
pub enum FieldKind {
    // Primitives (f_type 1-5, 8-9, 11-18)
    Bool,
    I8, I16, I32, I64,
    U8, U16, U32, U64,
    F32, F64,
    // Built-in objects (f_type 65-67)
    TString,
    TObject,
    TNamed,
    // Fixed-size array of primitive, count from f_array_length (f_type 20+x)
    FixedArray { elem: Box<FieldKind>, count: usize },
    // Variable-length array, count comes from another field at runtime (f_type 40+x)
    VariableArray { elem: Box<FieldKind>, count_field: String },
    // Object types
    Base(Arc<ClassSchema>),
    Object(Arc<ClassSchema>),
    ObjectPointer(Arc<ClassSchema>),
    StlVector(Box<FieldKind>),
    // Unrecognised / unsupported — reader will skip via byte_count
    Unknown,
}

