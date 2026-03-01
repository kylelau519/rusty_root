use crate::tobject::TObject;
use crate::utils;
use byteorder::{BigEndian, ReadBytesExt};
use std::fs::File;
use std::io;
use std::io::{BufReader, Seek, SeekFrom};

/*
* -Begin TNamed object (Base class of TStreamerInfo)
      6->9  ByteCount = Number of remaining bytes in TNamed object
                      |   OR'd with kByteCountMask (0x40000000)
     10->11 Version   = Version of TNamed Class
     12->21           = TObject object (Base class of TNamed) (see Format of the DATA for a TObject object).
                      |   Objects in StreamerInfo record are not referenced.
                      |   Would be two bytes longer (12->23) if object were referenced.
     22->.. fName     = Number of bytes in name of class that this TStreamerInfo object
                      |   describes, followed by the class name itself.  (TNamed::fName).
      0->.. fTitle    = Number of bytes in title of class that this TStreamerInfo object
                      |   describes, followed by the class title itself.  (TNamed::fTitle).
                      |  (Class title may be zero length)
 -End TNamed object
*/

#[derive(Debug, Default)]
pub struct TNamed {
    pub byte_count: u32,
    pub version: u16,
    pub tobject: TObject,
    pub l_name: u8,
    pub name: String,
    pub l_title: u8,
    pub title: String,
}

impl TNamed {
    pub fn read_tnamed_at(reader: &mut BufReader<File>, offset: u64) -> io::Result<Self> {
        reader.seek(std::io::SeekFrom::Start(offset))?;
        let byte_count = reader.read_u32::<byteorder::BigEndian>()?;
        let version = reader.read_u16::<byteorder::BigEndian>()?;
        let tobject = TObject::read_tobject(reader)?;
        let l_name = utils::read_u1(reader)?;
        let name = utils::read_string(reader, l_name as usize)?;
        let l_title = utils::read_u1(reader)?;
        let title = utils::read_string(reader, l_title as usize)?;
        Ok(TNamed {
            byte_count,
            version,
            tobject,
            l_name,
            name,
            l_title,
            title,
        })
    }

    pub fn read_tnamed(reader: &mut BufReader<File>) -> io::Result<Self> {
        let offset = reader.seek(SeekFrom::Current(0))?;
        Self::read_tnamed_at(reader, offset)
    }
}
