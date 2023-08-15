use std::io::{Read, Error, ErrorKind};

use byteorder::{ReadBytesExt, LittleEndian};

// See MS-ONESTORE 2.2.4.4. This is basically just a slice
// TODO: SHOULD FILECHUNKREFERENCE BE A GENERIC TYPE?
#[derive(Debug)]
pub struct FileChunkReference {
    pub start: u64,
    pub len: u64
}

impl FileChunkReference {

    pub fn from_reader<T: Read>(reader: &mut T, start_size_bits: u32, len_size_bits: u32) -> Result<FileChunkReference, Error> {
        let mut start: u64;
        let len: u64;

        match start_size_bits {
            8 => start = reader.read_u8()?.into(),
            16 => start = reader.read_u16::<LittleEndian>()?.into(),
            32 => start = reader.read_u32::<LittleEndian>()?.into(),
            64 => start = reader.read_u64::<LittleEndian>()?,
            _ => return Err(Error::new(ErrorKind::InvalidInput, "FCR start field must be 1, 2, 4, or 8 bytes"))
        }

        // If parsed start field is all ones, make our in-memory representation have all 1's. Used for is_nil()
        if start.count_ones() == start_size_bits {
            start = u64::MAX;
        }

        match len_size_bits {
            8 => len = reader.read_u8()?.into(),
            16 => len = reader.read_u16::<LittleEndian>()?.into(),
            32 => len = reader.read_u32::<LittleEndian>()?.into(),
            64 => len = reader.read_u64::<LittleEndian>()?,
            _ => return Err(Error::new(ErrorKind::InvalidInput, "FCR len field must be 1, 2, 4, or 8 bytes"))
        }

        Ok(FileChunkReference { start, len })
    }

    pub fn is_nil(&self) -> bool {
        self.start == std::u64::MAX && self.len == 0
    }

    pub fn is_zero(&self) -> bool {
        self.start == 0 && self.len == 0
    }
}