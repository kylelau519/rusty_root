use std::io;
use std::sync::Arc;


#[derive(Default, Debug)]
pub struct TList {
    pub byte_count: u32,
    pub version: u16,
    pub f_name_byte: u8,
    pub f_name: String,
    pub n_objects: u32,
    raw_byte_count: u32,
    // Optionally keep the raw payload for debugging
    pub decompressed_data: Option<Arc<[u8]>>,
    
}

pub struct TListObject {
    pub class_name: String,
    pub name: String,
    pub title: String,
    pub key_offset: u64,
    pub key_length: u32,
}
pub struct ReaderCursor<'a> {
    data: &'a Arc<[u8]>,
    position: usize,
}

impl<'a> ReaderCursor<'a> {
    pub fn new(data: &'a Arc<[u8]>) -> Self {
        Self { data, position: 0 }
    }

    pub fn read_u32(&mut self) -> Result<u32, io::Error> {
        if self.position + 4 <= self.data.len() {
            let buffer = &self.data[self.position..self.position + 4];
            let value = u32::from_be_bytes(buffer.try_into().map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "Failed to read u32"))?);
            self.position += 4;
            Ok(value)
        } else {
            Err(io::Error::from(io::ErrorKind::UnexpectedEof))
        }
    }

    pub fn read_u16(&mut self) -> Result<u16, io::Error> {
        if self.position + 2 <= self.data.len() {
            let buffer = &self.data[self.position..self.position + 2];
            let value = u16::from_be_bytes(buffer.try_into().map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "Failed to read u16"))?);
            self.position += 2;
            Ok(value)
        } else {
            Err(io::Error::from(io::ErrorKind::UnexpectedEof))
        }
    }

    pub fn read_u8(&mut self) -> Result<u8, io::Error> {
        if self.position < self.data.len() {
            let value = self.data[self.position];
            self.position += 1;
            Ok(value)
        } else {
            Err(io::Error::from(io::ErrorKind::UnexpectedEof))
        }
    }

    pub fn read_any(&mut self, length: usize) -> Result<&'a [u8], io::Error> {
        if self.position + length <= self.data.len() {
            let value = &self.data[self.position..self.position + length];
            self.position += length;
            Ok(value)
        } else {
            Err(io::Error::from(io::ErrorKind::UnexpectedEof))
        }
    }

    pub fn read_string(&mut self, length: usize) -> Result<String, io::Error> {
        if self.position + length <= self.data.len() {
            match String::from_utf8(self.data[self.position..self.position + length].to_vec()) {
                Ok(value) => {
                    self.position += length;
                    Ok(value)
                }
                Err(_) => Err(io::Error::new(io::ErrorKind::InvalidData, "Invalid UTF-8")),
            }
        } else {
            Err(io::Error::from(io::ErrorKind::UnexpectedEof))
        }
    }

    pub fn seek(&mut self, position: usize) -> bool {
        if position < self.data.len() {
            self.position = position;
            true
        } else {
            false
        }
    }

    pub fn get_position(&self) -> usize {
        self.position
    }

}

impl TList {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn new_from_data(data: Arc<[u8]>) -> Result<Self, io::Error> {
        let mut tlist = TList::new();
        tlist.decompressed_data = Some(data.clone());
        let mut cursor = ReaderCursor::new(tlist.decompressed_data.as_ref().unwrap());
        let raw_byte_count = cursor.read_u32()?;
        let byte_count = raw_byte_count & 0x3FFF_FFFF;
        let _has_bytecount = (raw_byte_count & 0x4000_0000) != 0;
        let _new_class = (raw_byte_count & 0x8000_0000) != 0; //https://root.cern/root/html520/src/TBufferFile.cxx.html?
        tlist.raw_byte_count = raw_byte_count;
        tlist.byte_count = byte_count;
        tlist.version = cursor.read_u16()?;
        let _tobject_version = cursor.read_u16()?;
        let _tobject_f_uniqueid = cursor.read_u32()?;
        let _tobject_f_bits = cursor.read_u32()?;
        tlist.f_name_byte = cursor.read_u8()?;
        tlist.f_name = cursor.read_string(tlist.f_name_byte as usize)?;
        tlist.n_objects = cursor.read_u32()?;
        Ok(tlist)
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
}