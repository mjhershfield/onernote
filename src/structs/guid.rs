pub type Guid = uuid::Uuid;
use std::io::{Read, Error};

pub use uuid::uuid as guid;

pub trait GuidExt {
    fn from_reader<T: Read>(reader: &mut T) -> Result<Guid, Error>;
}

impl GuidExt for Guid {
    fn from_reader<T: Read>(reader: &mut T) -> Result<Guid, Error> {
        let mut buf: [u8; 16] = [0; 16];
        reader.read_exact(&mut buf)?;
        Ok(Guid::from_bytes_le(buf))
    }
}