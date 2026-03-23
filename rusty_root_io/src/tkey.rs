use crate::compression::CompressionAlgorithm;
use crate::tstring::TString;
use binrw::{BinRead, BinReaderExt, BinResult, Endian};
use std::fmt;
use std::io::{Cursor, Read, Seek, SeekFrom};
use std::sync::Arc;

/*
 * https://root.cern/doc/v638/tdirectory.html
 */

#[derive(Default)]
pub struct TKey {
    pub n_bytes: u32,
    pub version: u16,
    pub obj_len: u32,
    pub datime: u32,
    pub key_len: u16,
    pub cycle: u16,
    pub seek_key: u64,
    pub seek_p_dir: u64,
    pub class_name: TString,
    pub name: TString,
    pub title: TString,
}
impl TKey {
    pub fn new() -> Self {
        TKey {
            n_bytes: 0,
            version: 0,
            obj_len: 0,
            datime: 0,
            key_len: 0,
            cycle: 0,
            seek_key: 0,
            seek_p_dir: 0,
            class_name: TString::default(),
            name: TString::default(),
            title: TString::default(),
        }
    }
    pub fn read_from<R: Read + Seek>(reader: &mut R, offset: u64) -> BinResult<Self> {
        reader.seek(std::io::SeekFrom::Start(offset))?;
        let key = TKey::read_be(reader)?;
        Ok(key)
    }
}
impl fmt::Debug for TKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("TKey")
            .field("n_bytes", &self.n_bytes)
            .field("version", &self.version)
            .field("obj_len", &self.obj_len)
            .field("datime", &self.datime)
            .field("key_len", &self.key_len)
            .field("cycle", &self.cycle)
            .field("seek_key", &self.seek_key)
            .field("seek_p_dir", &self.seek_p_dir)
            .field("class_name", &self.class_name.string)
            .field("name", &self.name.string)
            .field("title", &self.title.string)
            .finish()
    }
}

impl BinRead for TKey {
    type Args<'a> = ();

    fn read_options<R: Read + Seek>(
        reader: &mut R,
        endian: Endian,
        _args: Self::Args<'_>,
    ) -> BinResult<Self> {
        // 1. Read the fixed-width fields first
        let n_bytes: u32 = reader.read_type(endian)?;
        let version: u16 = reader.read_type(endian)?;
        let obj_len: u32 = reader.read_type(endian)?;
        let datime: u32 = reader.read_type(endian)?;
        let key_len: u16 = reader.read_type(endian)?;
        let cycle: u16 = reader.read_type(endian)?;

        // 2. THE FIX: Handle variable width pointers
        // ROOT Logic: if version > 1000, pointers are 64-bit
        let (seek_key, seek_p_dir) = if version > 1000 {
            let s_key: u64 = reader.read_type(endian)?;
            let s_pdir: u64 = reader.read_type(endian)?;
            (s_key, s_pdir)
        } else {
            let s_key: u32 = reader.read_type(endian)?;
            let s_pdir: u32 = reader.read_type(endian)?;
            (s_key as u64, s_pdir as u64) // Cast to u64 for struct parity
        };

        // 3. Read Strings using our parse_with logic or helper
        let class_name = TString::read_options(reader, endian, ())?;
        let name = TString::read_options(reader, endian, ())?;
        let title = TString::read_options(reader, endian, ())?;

        Ok(TKey {
            n_bytes,
            version,
            obj_len,
            datime,
            key_len,
            cycle,
            seek_key,
            seek_p_dir,
            class_name,
            name,
            title,
        })
    }
}

impl TKey {
    // the payload is deserialized, there's no way to correctly read the payload without the tkey header
    // Decompress the payload and return a Cursor over the combined key data and decompressed payload
    pub fn decompress_full<R: Read + Seek>(&self, reader: &mut R) -> BinResult<Cursor<Arc<[u8]>>> {
        let mut key_data = vec![0u8; self.key_len as usize];
        reader.seek(SeekFrom::Start(self.seek_key))?;
        reader.read_exact(&mut key_data)?;

        let compressed_size = self.n_bytes - u32::from(self.key_len);
        let mut compressed_data = vec![0u8; compressed_size as usize];
        reader.read_exact(&mut compressed_data)?;

        let decompressed_payload = CompressionAlgorithm::decompress(&compressed_data)?;
        assert_eq!(decompressed_payload.len(), self.obj_len as usize);

        let mut combined = key_data;
        combined.extend_from_slice(&decompressed_payload);
        return Ok(Cursor::new(Arc::from(combined)));
    }

    // Move the seek to the key's data and then call decompress_full
    pub fn decompress_full_from<R: Read + Seek>(
        &self,
        reader: &mut R,
        offset: u64,
    ) -> BinResult<Cursor<Arc<[u8]>>> {
        reader.seek(SeekFrom::Start(offset))?;
        self.decompress_full(reader)
    }

    // A helper function to read the decompressed payload instead
    pub fn read_from_payload<T, R>(
        reader: &mut R,
        endian: Endian,
        args: (&Self,), // Pass the header in so we can decompress
    ) -> BinResult<T>
    where
        T: for<'a> BinRead<Args<'a> = ()>,
        R: Read + Seek,
    {
        let (header,) = args;

        let mut combined_cursor = header.decompress_full(reader)?;
        combined_cursor.seek(SeekFrom::Start(header.key_len as u64))?; // Skip the key data to position at the payload

        T::read_options(&mut combined_cursor, endian, ())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::decode_datime;
    #[test]
    fn test_decode_datime() {
        let mut key = TKey::new();
        key.datime = 2054579214;
        assert_eq!(decode_datime(key.datime), "2025-09-27 06:16:14");
    }

    use binrw::BinRead;
    use std::fs::File;
    use std::io::SeekFrom;

    #[test]
    fn test_read_key() {
        let path =
            "/Users/kylelau519/Programming/rusty_root/rusty_root_io/testfiles/wzqcd_mc20a.root";
        let key_list_offset = 80365942;
        let file = File::open(path).expect("Failed to open ROOT file");
        let mut reader = std::io::BufReader::new(file);
        reader
            .seek(SeekFrom::Start(key_list_offset))
            .expect("Failed to seek to key list offset");

        let key = TKey::read_be(&mut reader).expect("Failed to read TKey with BinRead");
        dbg!(&key);
        assert_eq!(key.class_name, "TFile");
        assert_eq!(key.name, "user.holau.700590.Sh_2212_llvvjj_ss.e8433_s3681_r13167_r13146_p6697.46550259._000001.output.root");
    }
}
