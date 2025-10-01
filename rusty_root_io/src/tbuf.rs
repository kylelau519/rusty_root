use std::{io, sync::Arc};

pub struct TBuf<'a> {
    data: &'a Arc<[u8]>,
    position: usize,
}

impl<'a> TBuf<'a> {
    pub fn new(data: &'a Arc<[u8]>) -> Self {
        Self { data, position: 0 }
    }

    pub fn read_tstring(&mut self) -> Result<String, io::Error> {
        // ROOT TString: if first byte == 0xFF, next 4 bytes are length (big-endian); otherwise first byte is length
        let len_tag = self.read_u8()?;
        let len = if len_tag == 0xFF {
            self.read_u32()? as usize
        } else {
            len_tag as usize
        };
        self.read_string(len)
    }

    pub fn skip(&mut self, n: usize) -> Result<(), io::Error> {
        let new_pos = self.position.checked_add(n).ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "overflow in skip"))?;
        if new_pos <= self.data.len() {
            self.position = new_pos;
            Ok(())
        } else {
            Err(io::Error::from(io::ErrorKind::UnexpectedEof))
        }
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

    pub fn read_bytes(&mut self, length: usize) -> Result<&'a [u8], io::Error> {
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

    pub fn read_cstring(&mut self, cap: usize) -> Result<String, io::Error> {
        let start_pos = self.position;
        let end_pos = (start_pos + cap).min(self.data.len());
        for i in start_pos..end_pos {
            if self.data[i] == 0 {
                let s = String::from_utf8(self.data[start_pos..i].to_vec()).map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "Invalid UTF-8"))?;
                self.position = i + 1; // move past null terminator
                return Ok(s);
            }
        }
        Err(io::Error::new(io::ErrorKind::InvalidData, "No null terminator found within cap"))
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
