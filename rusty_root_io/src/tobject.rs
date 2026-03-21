use binrw::{BinRead, BinReaderExt, BinResult, Endian};
use std::io::{Read, Seek};

use crate::constant::K_IS_REFERENCED;
/*
* TObject
* https://root.cern/doc/v638/tobject.html
*/

#[derive(Debug, Default)]
pub struct TObject {
    pub version: u16,
    pub f_uniqueid: u32,
    pub f_bits: u32,
    pub pidf: u16,
}

impl BinRead for TObject {
    type Args<'a> = ();

    fn read_options<R: Read + Seek>(
        reader: &mut R,
        endian: Endian, // This is passed from the parent caller
        _args: Self::Args<'_>,
    ) -> BinResult<Self> {
        // Use read_type::<T>(endian) to stay flexible
        let version: u16 = reader.read_type(endian)?;
        let f_uniqueid: u32 = reader.read_type(endian)?;
        let f_bits: u32 = reader.read_type(endian)?;

        // Conditional logic remains manual but respects endianness
        let pidf = if (f_bits & K_IS_REFERENCED) != 0 {
            reader.read_type(endian)?
        } else {
            0
        };

        Ok(TObject {
            version,
            f_uniqueid,
            f_bits,
            pidf,
        })
    }
}
