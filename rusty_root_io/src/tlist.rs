use crate::tobject::TObject;
use crate::utils;
use crate::utils::binrw_read_string;
use binrw::io::{Read, Seek, SeekFrom};
use binrw::BinRead;
use byteorder::ReadBytesExt;

#[binrw::binread]
#[derive(Default, Debug)]
pub struct TList<T>
where
    // 1. T must own its data (no temporary references)
    T: BinRead + 'static,
    // 2. T must be readable with no arguments for any lifetime 'a
    for<'a> T: BinRead<Args<'a> = ()>,
{
    pub byte_count: u32,
    pub version: u16,
    pub tobject: TObject,
    pub f_name_byte: u8,
    #[br(parse_with = binrw_read_string, args(f_name_byte))]
    pub f_name: String,
    pub n_objects: u32,
    #[br(count = 1)]
    pub objects: Vec<T>,
}

impl<T> TList<T>
where
    // 1. T must own its data (no temporary references)
    T: BinRead + 'static,
    // 2. T must be readable with no arguments for any lifetime 'a
    for<'a> T: BinRead<Args<'a> = ()>,
{
    pub fn read_tlist_at<R: Read + Seek>(reader: &mut R, offset: u64) -> std::io::Result<Self> {
        reader.seek(SeekFrom::Start(offset))?;
        let byte_count = reader.read_u32::<byteorder::BigEndian>()?;
        let version = reader.read_u16::<byteorder::BigEndian>()?;
        let tobject = TObject::read_tobject(reader)?;
        let f_name_byte = reader.read_u8()?;
        let f_name = utils::read_string(reader, f_name_byte as usize)?;
        let n_objects = reader.read_u32::<byteorder::BigEndian>()?;
        Ok(TList {
            byte_count,
            version,
            tobject,
            f_name_byte,
            f_name,
            n_objects,
            objects: Vec::new(), // Placeholder, as we don't know the type T or how to read it yet
        })
    }
    pub fn read_tlist<R: Read + Seek>(reader: &mut R) -> std::io::Result<Self> {
        let loc = reader.seek(SeekFrom::Current(0))?;
        Self::read_tlist_at(reader, loc)
    }

    pub fn read_tlist_metadata_at<R: Read + Seek>(
        reader: &mut R,
        offset: u64,
    ) -> std::io::Result<Self> {
        reader.seek(SeekFrom::Start(offset))?;
        let byte_count =
            reader.read_u32::<byteorder::BigEndian>()? & crate::constant::K_BYTECOUNTMASK;
        let version = reader.read_u16::<byteorder::BigEndian>()?;
        let tobject = TObject::read_tobject(reader)?;
        let f_name_byte = reader.read_u8()?;
        let f_name = utils::read_string(reader, f_name_byte as usize)?;
        let n_objects = reader.read_u32::<byteorder::BigEndian>()?;
        Ok(TList {
            byte_count,
            version,
            tobject,
            f_name_byte,
            f_name,
            n_objects,
            objects: Vec::new(), // Placeholder, as we don't know the type T or how to read it yet
        })
    }

    pub fn read_tlist_metadata<R: Read + Seek>(reader: &mut R) -> std::io::Result<Self> {
        let loc = reader.seek(SeekFrom::Current(0))?;
        Self::read_tlist_metadata_at(reader, loc)
    }
}
