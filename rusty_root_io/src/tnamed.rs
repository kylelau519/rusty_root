use crate::constant::K_BYTECOUNTMASK;
use crate::tobject::TObject;
use crate::tstring::TString;
use binrw::io::{Read, Seek, SeekFrom};
use binrw::{binread, BinRead, BinResult};

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
#[binread]
#[br(big)]
#[derive(Debug, Default)]
pub struct TNamed {
    #[br(map = |x: u32| x & K_BYTECOUNTMASK)]
    pub byte_count: u32,
    pub version: u16,
    pub tobject: TObject,
    pub name: TString,
    pub title: TString,
}

impl TNamed {
    pub fn read_from<R: Read + Seek>(reader: &mut R, offset: u64) -> BinResult<Self> {
        reader.seek(SeekFrom::Start(offset))?;
        Self::read_options(reader, binrw::Endian::Big, ())
    }
}
