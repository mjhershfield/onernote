mod structs;

use std::fs::File;
use std::io::BufReader;
use std::path::Path;

use crate::structs::header::OneNoteFileHeader;
use crate::structs::transactionlog::TransactionLog;
use crate::structs::{FromFileChunk, ListFromFileChunk};
use crate::structs::filechunkreference::FileChunkReference;
use crate::structs::filenodelist::FileNodeList;

// TODO: Spin this function off into a separate OneStore::parse() function
fn main() -> Result<(), std::io::Error> {
    let path = Path::new("./scratchu23.one");
    let display = path.display();

    let file = match File::open(&path) {
        Err(why) => panic!("couldn't open {}: {}", display, why),
        Ok(file) => file,
    };

    let mut reader = BufReader::new(file);
    let start_of_file: FileChunkReference = FileChunkReference { start: 0, len: 1024 };

    // Read header from beginning of document
    let header: OneNoteFileHeader = OneNoteFileHeader::from_reader(&start_of_file, &mut reader)?;
    println!("{:#?}", header);

    // Read transaction log based on fcr and length given in header
    let transaction_log_fcr: FileChunkReference = header.transaction_log;
    let transaction_list_len: u64 = header.transactions_in_log.into();
    let transaction_log: TransactionLog = TransactionLog::from_reader(&transaction_log_fcr, &mut reader, transaction_list_len)?;
    println!("{:#?}", transaction_log);

    // Number of file nodes in the list is given by the transaction log entry for this list
    let file_node_list_root_fcr: FileChunkReference = header.file_node_list_root;
    let file_node_list_root = FileNodeList::from_reader(&file_node_list_root_fcr, &mut reader, &transaction_log).unwrap();
    println!("{:#?}", file_node_list_root);

    // Can we read the root object space?
    // TODO: Parse the revision manifest lists for an object space.
    let object_space_1_fcr = &file_node_list_root.file_nodes[0].file_chunk_ref;
    let object_space_1 = FileNodeList::from_reader(object_space_1_fcr, &mut reader, &transaction_log)?;
    println!("{:#?}", object_space_1);

    // Read latest revision
    let revision_manifest_fcr = &object_space_1.file_nodes[1].file_chunk_ref;
    let revision_manifest = FileNodeList::from_reader(revision_manifest_fcr, &mut reader, &transaction_log)?;
    println!("{:#?}", revision_manifest);

    Ok(())

}