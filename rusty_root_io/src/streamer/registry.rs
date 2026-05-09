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
    TString,
    Base(Arc<ClassSchema>),   // recurse into base class fields
    Object(Arc<ClassSchema>), // embedded object
    ObjectPointer(Arc<ClassSchema>),
    FixedArray { elem: Box<FieldKind>, count: usize },
    StlVector(Box<FieldKind>),
}
