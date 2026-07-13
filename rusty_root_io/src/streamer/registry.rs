use binrw::io::{Read, Seek, SeekFrom};
use binrw::{BinRead, BinReaderExt, BinResult, Endian};
use std::collections::HashMap;
use std::{fmt, sync::Arc};

use crate::objects::tnamed::TNamed;
use crate::objects::tobject::TObject;
use crate::objects::tstring::TString;

pub trait ReadSeek: Read + Seek {}
impl<T: Read + Seek> ReadSeek for T {}

pub struct ClassSchema {
    pub name: String,
    pub version: u16,
    pub checksum: u32,
    pub fields: Vec<FieldSchema>, // ordered, base class fields first
    pub plan: Vec<CompiledReader>,
}

pub struct FieldSchema {
    pub name: String,
    pub kind: FieldKind,
}

#[derive(Clone)]
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
    // Built-in objects (f_type 65-67)
    TString,
    TObject,
    TNamed,
    // Fixed-size array of primitive, count from f_array_length (f_type 20+x)
    FixedArray {
        elem: Box<FieldKind>,
        count: usize,
    },
    // Variable-length array, count comes from another field at runtime (f_type 40+x)
    VariableArray {
        elem: Box<FieldKind>,
        count_field: String,
    },
    // Object types
    Base(Arc<ClassSchema>),
    Object(Arc<ClassSchema>),
    ObjectPointer(Arc<ClassSchema>),
    StlVector(Box<FieldKind>),
    // Unrecognised / unsupported — reader will skip via byte_count
    Unknown,
}
// Arc, not Box: a Base/Object field reuses the nested schema's already-built
// plan wholesale (see register_tstreamerinfo), which means cloning entries out
// of it. Arc::clone is a refcount bump regardless of the closure inside; Box
// has no such Clone impl since it can't generically duplicate the boxed value.
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
        FieldKind::VariableArray { elem, count_field } => {
            todo!("VariableArray reader not implemented yet")
        }
        FieldKind::Base(class_schema) => todo!("Base reader not implemented yet"),
        FieldKind::Object(class_schema) => todo!("Object reader not implemented yet"),
        FieldKind::ObjectPointer(class_schema) => todo!("ObjectPointer reader not implemented yet"),
        FieldKind::StlVector(field_kind) => todo!("StlVector reader not implemented yet"),
        FieldKind::Unknown => todo!("Unknown field reader not implemented yet"),
    }
}

// Manual, indentation-based Debug impls below: the derived output nests every
// Base/Object/ObjectPointer schema inline, which turns even a small class like
// TH1F into thousands of lines of struct/enum boilerplate. These instead print
// one line per field in a flat, indented tree.

fn write_indent(f: &mut fmt::Formatter<'_>, indent: usize) -> fmt::Result {
    for _ in 0..indent {
        write!(f, "  ")?;
    }
    Ok(())
}

impl ClassSchema {
    fn write_tree(&self, f: &mut fmt::Formatter<'_>, indent: usize) -> fmt::Result {
        writeln!(
            f,
            "{} v{} (checksum={})",
            self.name, self.version, self.checksum
        )?;
        for field in &self.fields {
            field.write_tree(f, indent + 1)?;
        }
        Ok(())
    }
}

impl fmt::Debug for ClassSchema {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.write_tree(f, 0)
    }
}

impl FieldSchema {
    fn write_tree(&self, f: &mut fmt::Formatter<'_>, indent: usize) -> fmt::Result {
        write_indent(f, indent)?;
        write!(f, "{}: ", self.name)?;
        self.kind.write_tree(f, indent)
    }
}

impl fmt::Debug for FieldSchema {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.write_tree(f, 0)
    }
}

impl FieldKind {
    // Multi-line form used for a field's own line; recurses into nested
    // schemas as an indented sub-tree.
    fn write_tree(&self, f: &mut fmt::Formatter<'_>, indent: usize) -> fmt::Result {
        match self {
            FieldKind::FixedArray { elem, count } => {
                write!(f, "[")?;
                elem.write_inline(f)?;
                writeln!(f, "; {count}]")
            }
            FieldKind::VariableArray { elem, count_field } => {
                write!(f, "[")?;
                elem.write_inline(f)?;
                writeln!(f, "; {count_field}]")
            }
            FieldKind::Base(schema) => {
                writeln!(
                    f,
                    "Base -> {} v{} (checksum={})",
                    schema.name, schema.version, schema.checksum
                )?;
                for field in &schema.fields {
                    field.write_tree(f, indent + 1)?;
                }
                Ok(())
            }
            FieldKind::Object(schema) => {
                writeln!(
                    f,
                    "Object -> {} v{} (checksum={})",
                    schema.name, schema.version, schema.checksum
                )?;
                for field in &schema.fields {
                    field.write_tree(f, indent + 1)?;
                }
                Ok(())
            }
            FieldKind::ObjectPointer(schema) => {
                writeln!(
                    f,
                    "ObjectPointer -> {} v{} (checksum={})",
                    schema.name, schema.version, schema.checksum
                )?;
                for field in &schema.fields {
                    field.write_tree(f, indent + 1)?;
                }
                Ok(())
            }
            FieldKind::StlVector(elem) => {
                write!(f, "StlVector<")?;
                elem.write_inline(f)?;
                writeln!(f, ">")
            }
            _ => {
                self.write_inline(f)?;
                writeln!(f)
            }
        }
    }

    // Single-line, no-recursion form used for array/vector element types.
    fn write_inline(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FieldKind::Bool => write!(f, "Bool"),
            FieldKind::I8 => write!(f, "I8"),
            FieldKind::I16 => write!(f, "I16"),
            FieldKind::I32 => write!(f, "I32"),
            FieldKind::I64 => write!(f, "I64"),
            FieldKind::U8 => write!(f, "U8"),
            FieldKind::U16 => write!(f, "U16"),
            FieldKind::U32 => write!(f, "U32"),
            FieldKind::U64 => write!(f, "U64"),
            FieldKind::F32 => write!(f, "F32"),
            FieldKind::F64 => write!(f, "F64"),
            FieldKind::TString => write!(f, "TString"),
            FieldKind::TObject => write!(f, "TObject"),
            FieldKind::TNamed => write!(f, "TNamed"),
            FieldKind::FixedArray { elem, count } => {
                write!(f, "[")?;
                elem.write_inline(f)?;
                write!(f, "; {count}]")
            }
            FieldKind::VariableArray { elem, count_field } => {
                write!(f, "[")?;
                elem.write_inline(f)?;
                write!(f, "; {count_field}]")
            }
            FieldKind::Base(schema) => write!(f, "Base<{}>", schema.name),
            FieldKind::Object(schema) => write!(f, "Object<{}>", schema.name),
            FieldKind::ObjectPointer(schema) => write!(f, "ObjectPointer<{}>", schema.name),
            FieldKind::StlVector(elem) => {
                write!(f, "StlVector<")?;
                elem.write_inline(f)?;
                write!(f, ">")
            }
            FieldKind::Unknown => write!(f, "Unknown"),
        }
    }
}

impl fmt::Debug for FieldKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.write_inline(f)
    }
}

#[derive(Debug)]
pub enum FieldValue {
    Bool(bool),
    I8(i8),
    I16(i16),
    I32(i32),
    I64(i64),
    U8(u8),
    U16(u16),
    U32(u32),
    U64(u64),
    F32(f32),
    F64(f64),
    TString(TString),
    TObject(TObject),
    TNamed(TNamed),
    Array(Vec<FieldValue>),
    Object(ClassSchema),
    ObjectPointer(Option<Box<ClassSchema>>),
    StlVector(Vec<FieldValue>),
    // A field we couldn't read with full fidelity (e.g. unresolved back-reference,
    // unmodelled STL element kind). Carries a reason for debugging.
    Unsupported(&'static str),
}
