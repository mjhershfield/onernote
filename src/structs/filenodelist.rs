use byteorder::{LittleEndian, ReadBytesExt};

use super::{filenode::*, filechunkreference::FileChunkReference, transactionlog::TransactionLog};
use std::io::{Read, Seek, SeekFrom, Error, ErrorKind};

pub const FILE_NODE_LIST_HEADER_MAGIC: u64 = 0xA4567AB1F5F7F4C4;
pub const FILE_NODE_LIST_FOOTER_MAGIC: u64 = 0x8BC215C38233BA4B;

#[derive(Debug)]
pub struct FileNodeList {
    pub id: u32,
    pub fragment_sequence_index: u32,
    pub file_nodes: Vec<FileNode>
}

// TODO: CONVERT FILE NODE LISTS TO ITERATORS?
// Maybe make the file node list some kind of DFS tree iterator with a find by GUID function to search for specific nodes?

impl FileNodeList {
    pub fn from_reader<T: Read + Seek>(fcr: &FileChunkReference, reader: &mut T, transaction_log: &TransactionLog) -> Result<Self, std::io::Error> where Self: Sized {
        reader.seek(SeekFrom::Start(fcr.start))?;

        // Parse file node list header
        let magic = reader.read_u64::<LittleEndian>()?;
        if magic != FILE_NODE_LIST_HEADER_MAGIC {
            return Err(Error::new(ErrorKind::InvalidData, "Invalid file node header magic"));
        }

        let id = reader.read_u32::<LittleEndian>()?;
        let fragment_sequence_index = reader.read_u32::<LittleEndian>()?;

        let mut file_node_list = FileNodeList{
            id,
            fragment_sequence_index,
            file_nodes: Vec::new()
        };

        // Get length of this file node list from the transaction log
        let file_node_list_len: u32 = *transaction_log.get(&file_node_list.id).unwrap();
        let mut current_file_node: u32 = 0;

        while current_file_node < file_node_list_len && reader.stream_position()? < fcr.start + fcr.len - 20 {
            let new_file_node = FileNode::from_reader(reader)?;
            file_node_list.file_nodes.push(new_file_node);
            current_file_node += 1;
        }

        // Verify nextFragment file chunk reference always or only if needed?
        if current_file_node < file_node_list_len {
            todo!("Move on to next fragment of file node list");
        }

        // Verify footer
        reader.seek(SeekFrom::Start(fcr.start + fcr.len - 20))?;
        let _next_fragment = FileChunkReference::from_reader(reader, 64, 32)?;
        let footer = reader.read_u64::<LittleEndian>()?;
        if footer != FILE_NODE_LIST_FOOTER_MAGIC {
            return Err(Error::new(ErrorKind::InvalidData, "Incorrect footer magic for FileNodeListFragment"));
        }

        Ok(file_node_list)
    }
}