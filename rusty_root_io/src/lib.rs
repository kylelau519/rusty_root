use std::fs::File;
use std::io::{BufReader, Read, SeekFrom};
use std::io;
use byteorder::{BigEndian, ReadBytesExt};

/*
    The first data record starts at byte fBEGIN (currently set to kBEGIN).
    Bytes 1->kBEGIN contain the file description. When fVersion >= 1000000,
    it is a large file (> 2 GB) and the offsets will be 8 bytes long and
    fUnits will be set to 8:

    Byte Range      | Record Name   | Description
    --------------- | ------------- | -----------------------------------------------
    1->4            | "root"        | Root file identifier
    5->8            | fVersion      | File format version
    9->12           | fBEGIN        | Pointer to first data record
    13->16 [13->20] | fEND          | Pointer to first free word at the EOF -->8
    17->20 [21->28] | fSeekFree     | Pointer to FREE data record -->8
    21->24 [29->32] | fNbytesFree   | Number of bytes in FREE data record
    25->28 [33->36] | nfree         | Number of free data records
    29->32 [37->40] | fNbytesName   | Number of bytes in TNamed at creation time
    33->33 [41->41] | fUnits        | Number of bytes for file pointers
    34->37 [42->45] | fCompress     | Compression level and algorithm
    38->41 [46->53] | fSeekInfo     | Pointer to TStreamerInfo record --> 8
    42->45 [54->57] | fNbytesInfo   | Number of bytes in TStreamerInfo record
    46->63 [58->75] | fUUID         | Universal Unique ID
*/
enum HeaderPtrWidth {
    Off32,
    Off64,
}

impl HeaderPtrWidth {
    fn new(version: u32) -> Self {
        if version >= 1_000_000 {
            HeaderPtrWidth::Off64
        } else {
            HeaderPtrWidth::Off32
        }
    }
}

#[derive(Debug)]
pub struct TFileHeader {
    f_version: u32,
    f_begin: u32,
    f_end: u64,
    f_seek_free: u64,
    f_nbytes_free: u32,
    n_free: u32,
    f_nbytes_name: u32,
    f_units: u8,
    f_compress: i32,
    f_seek_info: u64,
    f_nbytes_info: u32,
    f_uuid: [u8; 16],
}
#[derive(Debug)]
pub struct TFile {
    header: TFileHeader,
    // other fields...
}

impl TFile {
    pub fn read_header(path: &str) -> io::Result<Self> {
        let file = File::open(path)?;
        let mut reader = BufReader::new(file);

        // --- magic ---
        let mut magic = [0u8; 4];
        reader.read_exact(&mut magic)?; //pointer advance whenever you do read_exact and read_u##
        if &magic != b"root" {
            return Err(io::Error::new(io::ErrorKind::InvalidData, "not a ROOT file"));
        }
        // Minimal header (new & old formats share the start)
        let f_version = reader.read_u32::<BigEndian>()?; 
        let f_begin = reader.read_u32::<BigEndian>()?;

        let header_ptr_width = HeaderPtrWidth::new(f_version);
        let read_hdr_ptr = |r: &mut BufReader<File>| -> io::Result<u64> {
            match header_ptr_width {
                HeaderPtrWidth::Off64 => r.read_u64::<BigEndian>(),
                HeaderPtrWidth::Off32 => Ok(r.read_u32::<BigEndian>()? as u64),
            }
        };

        // Read the rest of the header fields in the documented order
        let f_end = read_hdr_ptr(&mut reader)?;
        let f_seek_free = read_hdr_ptr(&mut reader)?;
        let f_nbytes_free = reader.read_u32::<BigEndian>()?;
        let n_free = reader.read_u32::<BigEndian>()?;
        let f_nbytes_name = reader.read_u32::<BigEndian>()?;

        // fUnits (1 or 8)
        let mut units_buf = [0u8; 1];
        reader.read_exact(&mut units_buf)?;
        let f_units = units_buf[0];

        // fCompress (signed i32 in ROOT)
        let f_compress = reader.read_i32::<BigEndian>()?;

        // fSeekInfo, fNbytesInfo
        let f_seek_info = read_hdr_ptr(&mut reader)?;
        let f_nbytes_info = reader.read_u32::<BigEndian>()?;

        // fUUID (16 bytes)
        let mut f_uuid = [0u8; 16];
        reader.read_exact(&mut f_uuid)?;


        let header = TFileHeader {
            f_version,
            f_begin,
            f_end,
            f_seek_free,
            f_nbytes_free,
            n_free,
            f_nbytes_name,
            f_units,
            f_compress,
            f_seek_info,
            f_nbytes_info,
            f_uuid,
        };
        Ok(Self { header })
    }
}

// impl TFileHeader {
//     pub fn new() -> Self {
//         TFileHeader {
//             magic: [0; 4],
//             fVersion: 0,
//             fBEGIN: 0,
//             fEND: 0,
//             fSeekFree: 0,
//             fNbytesFree: 0,
//             nfree: 0,
//             fNbytesName: 0,
//             fUnits: 0,
//             fCompress: 0,
//             fSeekInfo: 0,
//             fNbytesInfo: 0,
//             fUUID: [0; 16],
//         }
//     }
// }

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_read_root_header() {
        let path = "/Users/kylelau519/Programming/rusty_root/rusty_root_io/testfiles/output.root";
        let file = TFile::read_header(path).expect("Failed to read ROOT header");
        dbg!(&file.header);
    }

}
