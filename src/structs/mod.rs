use std::io::{Read, Seek};

pub mod guid;
pub mod exguid;
pub mod header;
pub mod filechunkreference;
pub mod filenodelist;
pub mod filenode;
pub mod transactionlog;

use filechunkreference::FileChunkReference;

use self::filenode::{FileType, FileNode};

// TODO: Are these traits useless after the refactoring?
pub trait FromFileChunk {
    fn from_reader<T: Read + Seek>(fcr: &FileChunkReference, reader: &mut T) -> Result<Self, std::io::Error> where Self: Sized;
}

pub trait ListFromFileChunk {
    fn from_reader<T: Read + Seek>(fcr: &FileChunkReference, reader: &mut T, len: u64) -> Result<Self, std::io::Error> where Self: Sized;
}

// Implement this trait for both file node and file node list so we can Rc<> them inside the file node list object as next, child
// pub trait FileNodeListTrait {
//     fn next_node<T: FileNodeListTrait>(&self) -> Option<&FileNode>;
//     fn data(&self) -> Option<&FileType>;
//     fn child<T: FileNodeListTrait>(&self) -> Option<&T>;
// }