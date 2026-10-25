use std::collections::HashMap;
use std::sync::Arc;

use crate::objects::tkey::TKey;
use crate::objects::tlist::TList;
use crate::streamer::registry::{
    build_field_reader, ClassSchema, CompiledReader, FieldKind, FieldSchema,
};
use crate::streamer::tstreamer_element::TStreamerType;
use crate::streamer::tstreamerinfo::TStreamerInfo;
use binrw::io::{Read, Seek, SeekFrom};
use binrw::BinRead;

// https://root.cern/doc/v638/streamerinfo.html
#[binrw::binread]
#[derive(Debug, Default)]
pub struct StreamerInfo {
    pub streamer_info_header: TKey,
    // the last element of StreamerInfo is not TStreamerInfo but some object called "ListOfRules".. not written anywhere, we just skip for the sanity
    #[br(parse_with = TKey::read_from_payload, args(&streamer_info_header))]
    pub tlist: TList<TStreamerInfo>,
    #[br(ignore)]
    pub registry: HashMap<String, Arc<ClassSchema>>,
}

impl StreamerInfo {
    pub fn read_from<R: Read + Seek>(reader: &mut R, offset: u64) -> binrw::BinResult<Self> {
        reader.seek(SeekFrom::Start(offset))?;
        Self::read_options(reader, binrw::Endian::Big, ())
    }

    pub fn iter(&self) -> impl Iterator<Item = &TStreamerInfo> {
        self.tlist.iter()
    }
    // turning the TStreamerInfo into our rust usable ClassSchemas
    pub fn populate_registry(&mut self) {
        // Pass 1: index all TStreamerInfo by class name
        let tstreamerinfo_map: HashMap<&str, &TStreamerInfo> = self
            .tlist
            .iter()
            .map(|tsi| (tsi.tnamed.name.as_str(), tsi))
            .collect();

        // Pass 2: register each class, recursing into base classes first
        let names: Vec<String> = tstreamerinfo_map.keys().map(|s| s.to_string()).collect();
        for name in &names {
            register_tstreamerinfo(name, &tstreamerinfo_map, &mut self.registry);
        }
    }

    pub fn get_schema(&self, class_name: &str) -> Option<&Arc<ClassSchema>> {
        self.registry.get(class_name)
    }
}

fn build_plan(name: &str, registry: &mut HashMap<String, Arc<ClassSchema>>) {
    let mut plan: Vec<CompiledReader> = Vec::new();
    if let Some(schema) = registry.get(name) {
        if schema.plan_completed {
            return;
        }
        let fields = &schema.fields;
        fields.iter().for_each(|field| {
            let kind = &field.kind;
            let name = &field.name;
        });
        schema.plan = plan;
        schema.plan_completed = true;
    }
}

fn get_plan(
    name: &str,
    registry: &HashMap<String, Arc<ClassSchema>>,
) -> Option<Vec<CompiledReader>> {
    if let Some(schema) = registry.get(name) {
        if schema.plan_completed {
            return Some(schema.plan.clone());
        } else {
            return None;
        }
    }
    None
}
fn register_tstreamerinfo(
    name: &str,
    tstreamerinfo_map: &HashMap<&str, &TStreamerInfo>,
    registry: &mut HashMap<String, Arc<ClassSchema>>,
) {
    if registry.contains_key(name) {
        return;
    }

    let tsi = match tstreamerinfo_map.get(name) {
        Some(tsi) => tsi,
        None => return, // not in this file's StreamerInfo, skip
    };

    // Recurse into every embedded (non-pointer) dependency before processing
    // this one, so registry.get() lookups in to_field_kind never depend on
    // HashMap<_, _>::keys() iteration order. Safe for all of these because a
    // value member can never cycle back to its own containing class — unlike
    // TStreamerObjectPointer (left lazy/on-demand, since pointers can cycle).
    for elem in tsi.iter() {
        match &elem.tstreamer_type {
            TStreamerType::TStreamerBase { .. } => {
                let base_name = elem.tstreamer_element_base.tnamed.name.as_str();
                register_tstreamerinfo(base_name, tstreamerinfo_map, registry);
            }
            TStreamerType::TStreamerObject | TStreamerType::TStreamerObjectAny => {
                let class_name = elem.tstreamer_element_base.type_name.trim_end_matches('\0');
                register_tstreamerinfo(class_name, tstreamerinfo_map, registry);
            }
            TStreamerType::TStreamerBasicType => {
                let base = &elem.tstreamer_element_base;
                // f_type 61/62 (kObject/kAny): an embedded object member ROOT
                // serialized via the "basic type" descriptor instead of
                // TStreamerObject(Any).
                if base.f_type == 61 || base.f_type == 62 {
                    let class_name = base.type_name.trim_end_matches('\0');
                    register_tstreamerinfo(class_name, tstreamerinfo_map, registry);
                }
            }
            _ => {}
        }
    }

    let fields: Vec<FieldSchema> = tsi
        .iter()
        .map(|elem| FieldSchema {
            name: elem.tstreamer_element_base.tnamed.name.to_string(),
            kind: elem.to_field_kind(registry),
        })
        .collect();
    dbg!(&fields);
    let plan: Vec<CompiledReader> = Vec::new();
    let plan_completed = false;
    let schema = Arc::new(ClassSchema {
        name: name.to_string(),
        version: tsi.f_class_version as u16,
        checksum: tsi.f_checksum,
        fields,
        plan,
        plan_completed,
    });
    registry.insert(name.to_string(), schema);
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::io::{BufReader, Read, Seek, SeekFrom};
    #[test]
    fn test_streamer_info_header() {
        let path =
            "/Users/kylelau519/Programming/rusty_root/rusty_root_io/testfiles/wzqcd_mc20a.root";
        let streamer_info_offset = 80357582;
        let file = File::open(path).expect("Failed to open ROOT file");
        let mut reader = BufReader::new(file);
        let key = TKey::read_from(&mut reader, streamer_info_offset)
            .expect("Failed to read TKey at offset");
        assert_eq!(key.title, "Doubly linked list");
        dbg!(&key);
    }

    use crate::core::compression::CompressionAlgorithm;
    #[test]
    fn test_decode_streamer_info() {
        let path =
            "/Users/kylelau519/Programming/rusty_root/rusty_root_io/testfiles/wzqcd_mc20a.root";
        let streamer_info_offset = 80357582;
        let file = File::open(path).expect("Failed to open ROOT file");
        let mut reader = BufReader::new(file);

        let obj_len: usize = 30257;
        let n_bytes: usize = 8360;
        let key_len: usize = 64;
        let mut data = vec![0u8; n_bytes - key_len as usize];
        reader
            .seek(SeekFrom::Start(streamer_info_offset + key_len as u64))
            .expect("Failed to seek to compressed data");
        reader
            .read_exact(&mut data)
            .expect("Failed to read compressed data");

        // Decompress the data
        let decompressed_data =
            CompressionAlgorithm::decompress(&data).expect("Failed to decompress data");
        dbg!(&decompressed_data.len());

        assert_eq!(decompressed_data.len(), obj_len as usize);
    }

    use crate::objects::tlist::TList;
    use binrw::BinRead;
    #[test]
    fn test_read_all_streamer_info() {
        let path =
            "/Users/kylelau519/Programming/rusty_root/rusty_root_io/testfiles/wzqcd_mc20a.root";
        let streamer_info_offset = 80357582;
        let file = File::open(path).expect("Failed to open ROOT file");
        let mut reader = BufReader::new(file);

        let key = TKey::read_from(&mut reader, streamer_info_offset)
            .expect("Failed to read TKey at offset");
        assert_eq!(key.title, "Doubly linked list");
        let payload_len = (key.n_bytes - key.key_len as u32) as usize;

        let key_data = {
            let mut buf = vec![0u8; key.key_len as usize];
            reader
                .seek(SeekFrom::Start(streamer_info_offset))
                .expect("Failed to seek to TKey");
            reader
                .read_exact(&mut buf)
                .expect("Failed to read TKey data");
            buf
        };
        let compressed_data = {
            let mut buf = vec![0u8; payload_len];
            reader
                .seek(SeekFrom::Start(streamer_info_offset + key.key_len as u64))
                .expect("Failed to seek to compressed data");
            reader
                .read_exact(&mut buf)
                .expect("Failed to read compressed data");
            buf
        };
        // i forgot why it is Necessary to combine and read the key and decompressed together
        // my guess is there's some decompressed data point back to the tkey
        let decompressed_data =
            CompressionAlgorithm::decompress(&compressed_data).expect("Failed to decompress data");
        let mut combined_data = key_data;
        combined_data.extend_from_slice(&decompressed_data);
        let mut cursor = std::io::Cursor::new(combined_data);
        let _tkey: TKey =
            TKey::read_be(&mut cursor).expect("Failed to read TKey from combined data");
        let _tlist: TList<TStreamerInfo> =
            TList::read_be(&mut cursor).expect("Failed to read TList from decompressed data");
        dbg!(&_tlist);
    }
    #[test]
    fn test_read_streamer_with_tkey() {
        let path =
            "/Users/kylelau519/Programming/rusty_root/rusty_root_io/testfiles/wzqcd_mc20a.root";
        let streamer_info_offset = 80357582;
        let file = File::open(path).expect("Failed to open ROOT file");
        let mut reader = BufReader::new(file);

        let tkey = TKey::read_from(&mut reader, streamer_info_offset)
            .expect("Failed to read TKey at offset");
        dbg!(&tkey);

        let mut decompressed_cursor = tkey
            .decompress_full(&mut reader)
            .expect("Failed to decompress StreamerInfo data");
        decompressed_cursor
            .seek(SeekFrom::Start(tkey.key_len as u64))
            .expect("Failed to seek to decompressed data after TKey");
        let tlist = TList::<TStreamerInfo>::read_be(&mut decompressed_cursor)
            .expect("Failed to read TList of TStreamerInfo from decompressed data");
        dbg!(&tlist);
    }

    #[test]
    fn test_streamer_info_registry() {
        let path =
            "/Users/kylelau519/Programming/rusty_root/rusty_root_io/testfiles/wzqcd_mc20a.root";
        let streamer_info_offset = 80357582;
        let file = File::open(path).expect("Failed to open ROOT file");
        let mut reader = BufReader::new(file);

        let mut streamer_info = StreamerInfo::read_from(&mut reader, streamer_info_offset)
            .expect("Failed to read StreamerInfo");
        streamer_info.populate_registry();

        // Check that we can get the schema for a known class
        let registry = &streamer_info.registry;
        registry.iter().for_each(|(name, schema)| {
            println!(
                "Class: {}, version: {}, fields: {}",
                name,
                schema.version,
                schema.fields.len()
            );
        });

        let th1_schema = streamer_info
            .get_schema("TH1F")
            .expect("Failed to get schema for TH1F");

        dbg!(&th1_schema);
        // th1_schema.fields.iter().for_each(|field| {
        //     println!("Field: {}, kind: {:?}", field.name, field.kind);
        // });
    }
}
