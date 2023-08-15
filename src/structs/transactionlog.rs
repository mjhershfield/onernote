use byteorder::{LittleEndian, ReadBytesExt};

use super::{ListFromFileChunk, filechunkreference::FileChunkReference};
use std::{io::{SeekFrom, Read, Seek, Error, ErrorKind}, collections::HashMap};

pub type TransactionLog = HashMap<u32, u32>;

// TODO: KEEP TRACK OF HOW LONG THE FIRST NODE OF THE TRANSACTION LIST IS (FROM FILECHUNKREF) TO SEE
// IF WE NEED TO MOVE TO NEXT FILE CHUNK IN THE LIST.
// TODO: VERIFY CRC FOR EACH TRANSACTION
pub struct TransactionEntry {
    pub src_id: u32,
    pub transaction_entry_switch: u32
}

impl TransactionEntry {
    fn from_reader<T: Read + Seek>(reader: &mut T) -> Result<TransactionEntry, Error> {
        let src_id = reader.read_u32::<LittleEndian>()?;
        let transaction_entry_switch = reader.read_u32::<LittleEndian>()?;

        Ok(TransactionEntry { src_id, transaction_entry_switch })
    }
}

impl ListFromFileChunk for TransactionLog {
    fn from_reader<T: Read + Seek>(fcr: &FileChunkReference, reader: &mut T, len: u64) -> Result<Self, Error> where Self: Sized {
        reader.seek(SeekFrom::Start(fcr.start))?;
        let mut current_transaction: u64 = 0;
        let mut transaction_log = TransactionLog::new();
        let mut current_transaction_entry: TransactionEntry = TransactionEntry { src_id: 1, transaction_entry_switch: 0 };
        
        // Iterate until we read all transactions or we reach the end of this fragment
        while current_transaction < len && reader.stream_position()? < (fcr.start + fcr.len - 12) {
            current_transaction_entry = TransactionEntry::from_reader(reader)?;

            if current_transaction_entry.src_id == 1 {
                // Sentinel entry; end of transaction
                current_transaction += 1;
            }
            else {
                // Otherwise, update transaction log for this src_id
                let src_id_to_update = transaction_log.entry(current_transaction_entry.src_id).or_insert(0);
                *src_id_to_update = current_transaction_entry.transaction_entry_switch;
            }
        }

        if current_transaction_entry.src_id != 1 {
            return Err(Error::new(ErrorKind::InvalidData, "Final transaction entry of log must be a sentinel"));
        }

        if current_transaction < len {
            todo!("Read next fragment based on fcr at end of list!");
        }

        Ok(transaction_log)
    }
}