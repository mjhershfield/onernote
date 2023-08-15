use std::io::{Read, Seek};

pub mod guid;
pub mod exguid;
pub mod header;
pub mod filechunkreference;
pub mod filenodelist;
pub mod filenode;
pub mod transactionlog;

use filechunkreference::FileChunkReference;

// TODO: Are these traits useless after the refactoring?
pub trait FromFileChunk {
    fn from_reader<T: Read + Seek>(fcr: &FileChunkReference, reader: &mut T) -> Result<Self, std::io::Error> where Self: Sized;
}

pub trait ListFromFileChunk {
    fn from_reader<T: Read + Seek>(fcr: &FileChunkReference, reader: &mut T, len: u64) -> Result<Self, std::io::Error> where Self: Sized;
}