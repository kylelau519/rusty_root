use binrw::io::{Read, Seek};
use binrw::{BinRead, BinReaderExt, Endian};
use std::ops::Deref;

#[derive(Default, Debug)]
pub struct TString {
    pub l_string: u8,
    pub string: String,
}

impl TString {
    pub fn new() -> Self {
        Self {
            l_string: 0,
            string: String::new(),
        }
    }
}

impl BinRead for TString {
    type Args<'a> = ();

    fn read_options<R: Read + Seek>(
        reader: &mut R,
        endian: Endian,
        _args: Self::Args<'_>,
    ) -> binrw::BinResult<Self> {
        let l_string: u8 = reader.read_type(endian)?;
        let actual_length = if l_string == 255 {
            reader.read_type(endian)?
        } else {
            l_string as u32
        };

        if actual_length == 0 {
            return Ok(Self {
                l_string,
                string: String::new(),
            });
        }

        let mut data_bytes = vec![0u8; actual_length as usize];
        reader.read_exact(&mut data_bytes)?;
        let string = String::from_utf8_lossy(&data_bytes).into_owned();
        Ok(Self { l_string, string })
    }
}

impl Deref for TString {
    type Target = String;

    fn deref(&self) -> &Self::Target {
        &self.string
    }
}

impl PartialEq<&str> for TString {
    fn eq(&self, other: &&str) -> bool {
        &self.string == other
    }
}

impl PartialEq<TString> for &str {
    fn eq(&self, other: &TString) -> bool {
        self == &other.string
    }
}
