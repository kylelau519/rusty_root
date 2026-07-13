use crate::core::constant::K_BYTECOUNTMASK;
use crate::objects::tnamed::TNamed;
use crate::objects::tobject::TObject;
use crate::objects::tstring::TString;
use crate::streamer::registry::{ClassSchema, FieldKind};
use binrw::io::{Read, Seek, SeekFrom};
use binrw::{BinRead, BinReaderExt, BinResult, Endian};
use std::collections::HashMap;
use std::sync::Arc;

// Interpreter responsible to turn a streamerinfo into a binrw usable struct
// The interpreter struct should have read_options from binrw available
// Concretely, this should define a compile time reader for any type in the schema field.

// An object that actually store values according to a `ClassSchema`.
// Reader should be defined so that at run time the reader can use a compiled interpreter
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
    Object(InterpretedSchema),
    ObjectPointer(Option<Box<InterpretedSchema>>),
    StlVector(Vec<FieldValue>),
    // A field we couldn't read with full fidelity (e.g. unresolved back-reference,
    // unmodelled STL element kind). Carries a reason for debugging.
    Unsupported(&'static str),
}

#[derive(Debug)]
pub struct InterpretedSchema {
    pub schema: Arc<ClassSchema>,
    // why this cannot be a HashMap? Because the order of the fields matters, and we need to preserve the order of the fields as they are defined in the schema. The order is important for reading variable-length arrays, where the count field must be read before the array field?
    pub fields: Vec<(String, FieldValue)>,
}

impl InterpretedSchema {
    pub fn get(&self, name: &str) -> Option<&FieldValue> {
        self.fields.iter().find(|(n, _)| n == name).map(|(_, v)| v)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::objects::th1::TH1;
    use crate::objects::tkey::TKey;
    use crate::streamer::streamerinfo::StreamerInfo;
    use std::fs::File;
    use std::io::{BufReader, SeekFrom};

    #[test]
    fn test_read_th1f_with_hard_code_object() {
        let path =
            "/Users/kylelau519/Programming/rusty_root/rusty_root_io/testfiles/wzqcd_mc20a.root";
        let streamer_info_offset = 80357582;
        let key = TKey {
            n_bytes: 612,
            version: 4,
            obj_len: 1038,
            datime: 2054579217,
            key_len: 95,
            cycle: 1,
            seek_key: 79795566,
            seek_p_dir: 100,
            class_name: TString::new("TH1F"),
            name: TString::new("cflow_AnaMuons_Baseline_NOSYS"),
            title: TString::new("Object Cutflow: AnaMuons.Baseline"),
        };
        let file = File::open(path).expect("Failed to open ROOT file");
        let mut reader = BufReader::new(file);

        let mut streamer_info = StreamerInfo::read_from(&mut reader, streamer_info_offset)
            .expect("Failed to read StreamerInfo");
        streamer_info.populate_registry();

        let schema = streamer_info.get_schema("TH1F").expect("no TH1F schema");
        dbg!(&schema);
        let mut decompressed_cursor = key
            .decompress_full(&mut reader)
            .expect("Failed to decompress TH1F data");
        decompressed_cursor
            .seek(SeekFrom::Start(key.key_len as u64 + 6))
            .expect("seek failed");
        let th1 = TH1::read_be(&mut decompressed_cursor).expect("Failed to read TH1F object");
        dbg!(&th1);
    }
}
