use binrw::{BinRead, BinReaderExt, BinResult, Endian};
use byteorder::{BigEndian, ReadBytesExt};
use std::io;
use std::io::{Read, Seek, SeekFrom};

use crate::constant::K_IS_REFERENCED;
/*
* TObject
* https://root.cern/doc/v638/tobject.html
*
* --------
  0->1  Version   = Version of TObject Class
  2->5  fUniqueID = Unique ID of object.  Currently, unless this object is or was
                       | referenced by a TRef or TRefArray, or is itself a TRef or TRefArray,
                       | this field is not used by ROOT.
  6->9  fBits     = A 32 bit mask containing status bits for the object.
                       | The bits relevant to ROOTIO are:
                       | 0x00000001 - if object in a list can be deleted.
                       | 0x00000008 - if other objects may need to be deleted when this one is.
                       | 0x00000010 - if object is referenced by pointer to persistent object.
                       | 0x00002000 - if object ctor succeeded but object shouldn't be used
                       | 0x01000000 - if object is on Heap.
                       | 0x02000000 - if object has not been deleted.
 The "pidf" field below is present only if this TObject object (or an object inheriting
      from it) is referenced by a pointer to persistent object.
 10->11 pidf  = An identifier of the TProcessID record for the process that wrote the
                       | object. This identifier is an unsigned short.  The relevant record
                       | has a name that is the string "ProcessID" concatenated with the ASCII
                       | decimal representation of "pidf" (no leading zeros).  0 is a valid pidf.
-------
 No object in the StreamerInfo record will be a reference or referenced, and all objects
      are on the heap.  So, for each occurrence in the StreamerInfo record, fUniqueID will be 0,
      fBits will be 0x03000000, and pidf will be absent.
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
