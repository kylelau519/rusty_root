use std::{fmt, io};
use std::sync::Arc;
use crate::constant::{K_BYTECOUNTMASK, K_HAS_BYTECOUNT, K_NEW_CLASSBIT, K_NEWCLASSTAG, K_NULLTAG};
use crate::tbuf::TBuf;

#[derive(Default)]
pub struct TList {
    pub byte_count: u32,
    pub version: u16,
    pub f_name_byte: u8,
    pub f_name: String,
    pub n_objects: u32,
    pub header_end_pos: usize,
    raw_byte_count: u32,
    // Optionally keep the raw payload for debugging
    pub decompressed_data: Option<Arc<[u8]>>,
    
}
#[derive(Debug, Clone, Default)]
pub struct TListObject {
    pub class_name: String,
    pub name: String,
    pub title: String,
    pub key_offset: u64,
    pub key_length: u32,
}

#[derive(Debug, Clone)]
pub struct ObjectEnvelope {
    pub class_name: String,
    pub version: u16,
    pub byte_count: u32,
    pub body_offset: usize, // offset within decompressed_data to the start of the object's body (right after version)
}
impl TList {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn new_from_data(data: Arc<[u8]>) -> Result<Self, io::Error> {
        let mut tlist = TList::new();
        tlist.decompressed_data = Some(data.clone());
        let mut cursor = TBuf::new(tlist.decompressed_data.as_ref().unwrap());
        let raw_byte_count = cursor.read_u32()?;
        let byte_count = raw_byte_count & K_BYTECOUNTMASK;
        let _has_bytecount = (raw_byte_count & K_HAS_BYTECOUNT) != 0;
        let _new_class = (raw_byte_count & K_NEW_CLASSBIT) != 0; //https://root.cern/root/html520/src/TBuf.cxx.html?
        tlist.raw_byte_count = raw_byte_count;
        tlist.byte_count = byte_count;
        tlist.version = cursor.read_u16()?;
        let _tobject_version = cursor.read_u16()?;
        let _tobject_f_uniqueid = cursor.read_u32()?;
        let _tobject_f_bits = cursor.read_u32()?;
        tlist.f_name_byte = cursor.read_u8()?;
        tlist.f_name = cursor.read_string(tlist.f_name_byte as usize)?;
        tlist.n_objects = cursor.read_u32()?;
        tlist.header_end_pos = cursor.get_position();
        Ok(tlist)
    }

    /// Read the first object's envelope (immediately after the TList header) and return
    /// an `ObjectEnvelope` describing class name (if NEW_CLASS), version (handling member-wise 0xFFFF),
    /// byte_count (0 if no size header), and the offset to the start of the object's body.
    pub fn extract_first_envelope(&self) -> Result<ObjectEnvelope, io::Error> {

        let data_arc = self.decompressed_data.as_ref().ok_or_else(|| io::Error::new(io::ErrorKind::Other, "no decompressed data stored"))?;
        let mut c = TBuf::new(data_arc);
        if !c.seek(self.header_end_pos) {
            return Err(io::Error::new(io::ErrorKind::UnexpectedEof, "header_end_pos out of bounds"));
        }
        // Step 1: read first u32; it can be a size header or a tag directly
        let first = c.read_u32()?;
        println!("First u32: {:X}", first);
        let tag: u32;
        let mut byte_count: u32 = 0;
        let mut has_size_header = false;

        if (first & K_HAS_BYTECOUNT) != 0 { // size header present
            has_size_header = true;
            byte_count = first & K_BYTECOUNTMASK;
            tag = c.read_u32()?;
            // tag = c.read_u32()?;  // second word is the tag
        } else {
            tag = first;
        }
        // Step 2: interpret tag
        let mut class_name = String::new();
        if tag == K_NULLTAG {
            return Err(io::Error::new(io::ErrorKind::InvalidData, "first element is null tag"));
        }

        let mut is_new_class = false;
        println!("{:X}", tag);
        if (tag & K_NEW_CLASSBIT) != 0 {
            if tag == K_NEWCLASSTAG {
                class_name = c.read_cstring(80)?;
                is_new_class = true;
            } else {
                todo!();
            }
        } else {
            todo!();
        }

        // Step 3: read version or member-wise marker
        let marker_or_version = c.read_u16()?;
        let version: u16 = if marker_or_version == 0xFFFF {
            // member-wise: u32 mw_size then real u16 version
            let _mw_size = c.read_u32()?; // you can log/inspect if desired
            c.read_u16()?
        } else {
            marker_or_version
        };

        // Body starts immediately after the version field we just read
        let body_offset = c.get_position();
        dbg!(first, tag, class_name.as_str(), is_new_class, marker_or_version, version, byte_count, has_size_header, body_offset);
        Ok(ObjectEnvelope {
            class_name: if is_new_class { class_name } else { class_name },
            version,
            byte_count: if has_size_header { byte_count } else { 0 },
            body_offset,
        })
    }

    /// Return the byte offset of the first object's BODY (right after ReadVersion),
    /// along with its envelope bytecount and class name. This minimally parses the
    /// object envelope: [u32 bytecount|flags][u32 tag][optional class name],
    /// then consumes ReadVersion (u16 or 0xFFFF + u32 + u16) and returns the
    /// cursor positioned at the body start.
    pub fn extract_object_with_offset(&self, envelope_start: usize) -> Result<(u32, String, u16), io::Error> {
        let data_arc = self
            .decompressed_data
            .as_ref()
            .ok_or_else(|| io::Error::new(io::ErrorKind::Other, "no decompressed data stored"))?;
        let mut c = TBuf::new(data_arc);
        if !c.seek(self.header_end_pos) {
            return Err(io::Error::new(io::ErrorKind::UnexpectedEof, "header_end_pos out of bounds"));
        }
        c.seek(envelope_start);
        // Envelope: first u32 is bytecount|flags
        let first = c.read_u32()?;
        let byte_count = first & K_BYTECOUNTMASK; // e.g. 0x4000_018B -> 0x018B

        let class_name: String = String::new();
        let version: u16 = 0;
        dbg!(format!("first u32: {:X}", first));

        Ok((byte_count, class_name, version))
    }

}
// after so many tests im 100% sure there's an extra header in TList which is not mentioned in the docs

impl fmt::Debug for TList{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("TList")
            .field("byte_count", &self.byte_count)
            .field("version", &self.version)
            .field("f_name_byte", &self.f_name_byte)
            .field("f_name", &self.f_name)
            .field("n_objects", &self.n_objects)
            .field("header_end_pos", &self.header_end_pos)
            .field("raw_byte_count", &self.raw_byte_count)
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tlist_creation() {
        let data: Vec<u8> = vec![64, 0, 69, 105, 0, 5, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 24, 64, 0, 1, 184, 255, 255, 255, 255, 84, 83, 116, 114, 101, 97, 109, 101, 114, 73];
        let tlist = TList::new_from_data(Arc::from(data));
        assert!(tlist.is_ok());
    }

    #[test]
    fn test_reader_cursor() {
        let data: Vec<u8> = vec![64, 0, 69, 105, 0, 5, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 24, 64, 0, 1, 184, 255, 255, 255, 255, 84, 83, 116, 114, 101, 97, 109, 101, 114, 73];
        let data: Arc<[u8]> = Arc::from(data);
        let tlist = TList::new_from_data(data.clone()).unwrap();
        dbg!(&tlist);
        let mut cursor = TBuf::new(&data);
        cursor.seek(tlist.header_end_pos);
        let mut last_class: Option<String> = None;
        let raw = cursor.read_u32().unwrap();
        let _new_class_tag = (raw & 0xFFFF_FFFF) != 0;
        let byte_count = raw & 0x3FFF_FFFF;
        let _has_bytecount = (raw & 0x4000_0000) != 0;
        let new_class = (raw & 0x8000_0000) != 0;
        dbg!(raw, _new_class_tag, byte_count, new_class, _has_bytecount);
    }

    #[test]
    fn test_extract_first_envelope() {
        // Same synthetic bytes as other tests; this ensures the method runs without panicking.
        let data: Vec<u8> = vec![64, 0, 69, 105, 0, 5, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 24, 64, 0, 1, 184, 255, 255, 255, 255, 84, 83, 116, 114, 101, 97, 109, 101, 114, 73];
        let tl = TList::new_from_data(Arc::from(data)).unwrap();
        let env = tl.extract_first_envelope().unwrap();
        // byte_count was present in synthetic header
   
    }
}