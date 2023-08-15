use std::io::Error;
use std::io::ErrorKind;
use std::io::Read;
use std::io::Seek;
use std::io::SeekFrom;

use byteorder::LittleEndian;
use byteorder::ReadBytesExt;

use crate::structs::guid::*;
use crate::structs::filechunkreference::*;

use super::FromFileChunk;

const FILE_TYPE_ONE: Guid = guid!("7B5C52E4-D88C-4DA7-AEB1-5378D02996D3");
const FILE_TYPE_ONETOC2: Guid = guid!("43FF2FA1-EFD9-4C76-9EE2-10EA5722765F");

const VALID_FILE_FORMAT: Guid = guid!("109ADD3F-911B-49F5-A5D0-1791EDC8AED8");

const CODE_VERSION_ONE: u32 = 0x2A;
const CODE_VERSION_ONETOC2: u32 = 0x1B;

#[derive(PartialEq, Clone, Copy, Debug)]
pub enum OneNoteFileType {
    One,
    OneToc2
}

#[derive(Debug)]
pub struct OneNoteFileHeader {
    pub file_type: OneNoteFileType,
    pub file_guid: Guid,
    pub transactions_in_log: u32,
    pub ancestor_guid: Guid,
    pub hashed_chunk_list: FileChunkReference,
    pub transaction_log: FileChunkReference,
    pub file_node_list_root: FileChunkReference,
    pub free_chunk_list: FileChunkReference,
    pub expected_file_length: u64,
    pub free_space_in_free_chunk_list: u64,
    pub file_version: Guid,
    pub file_version_generation: u64,
}

impl FromFileChunk for OneNoteFileHeader {
    fn from_reader<T: Read + Seek>(fcr: &FileChunkReference, reader: &mut T) -> Result<OneNoteFileHeader, Error> {
        reader.seek(SeekFrom::Start(fcr.start))?;

        // Determine file type from GUID
        let file_type_guid = Guid::from_reader(reader)?;
        let file_type: OneNoteFileType;
        match file_type_guid {
            FILE_TYPE_ONE => file_type = OneNoteFileType::One,
            FILE_TYPE_ONETOC2 => file_type = OneNoteFileType::OneToc2,
            _ => return Err(Error::new(ErrorKind::InvalidData, "File type GUID is not ONE or ONETOC2"))
        }

        let file_guid = Guid::from_reader(reader)?;

        // Verify legacy file version and file format GUIDs
        {
            let legacy_file_version_guid = Guid::from_reader(reader)?;
            if !legacy_file_version_guid.is_nil() {
                return Err(Error::new(ErrorKind::InvalidData, "Legacy file version GUID must be zero"))
            }
 
            let file_format = Guid::from_reader(reader)?;
            if file_format != VALID_FILE_FORMAT {
                return Err(Error::new(ErrorKind::InvalidData, "Invalid file format GUID"))
            }
        }

        // Verify ffv___CodeThat____ToThisFile
        for _ in 0..4 {
            let code_number = reader.read_u32::<LittleEndian>()?;
            match (&file_type, code_number) {
                (OneNoteFileType::One, CODE_VERSION_ONE) => {},
                (OneNoteFileType::OneToc2, CODE_VERSION_ONETOC2) => {},
                _ => return Err(Error::new(ErrorKind::InvalidData, "Invalid code version"))
            }
        }

        {
            let legacy_free_chunk_list = FileChunkReference::from_reader(reader, 32, 32)?;
            if !legacy_free_chunk_list.is_zero() {
                return Err(Error::new(ErrorKind::InvalidData, "Legacy free chunk list must equal 0"));
            }

            let legacy_transaction_log = FileChunkReference::from_reader(reader, 32, 32)?;
            if !legacy_transaction_log.is_nil() {
                return Err(Error::new(ErrorKind::InvalidData, "Legacy transaction log must equal nil"));
            }
        }

        let transactions_in_log = reader.read_u32::<LittleEndian>()?;
        if transactions_in_log == 0 {
            return Err(Error::new(ErrorKind::InvalidData, "Zero transactions in transaction log"));
        }

        {
            let legacy_expected_file_length = reader.read_u32::<LittleEndian>()?;
            if legacy_expected_file_length != 0 {
                return Err(Error::new(ErrorKind::InvalidData, "Legacy expected file length must equal 0"));
            }

            let placeholder = reader.read_u64::<LittleEndian>()?;
            if placeholder != 0 {
                return Err(Error::new(ErrorKind::InvalidData, "Placeholder 1 must equal 0"));
            }

            let legacy_file_node_list_root = FileChunkReference::from_reader(reader, 32, 32)?;
            if !legacy_file_node_list_root.is_nil() {
                return Err(Error::new(ErrorKind::InvalidData, "Legacy transaction log must equal nil"));
            }
            
            // Next 8 bytes must be ignored
            let _ = reader.read_u64::<LittleEndian>()?;
        }

        let ancestor_guid = Guid::from_reader(reader)?;

        {
            // crcName field... do we have to check this?
            let _ = reader.read_u32::<LittleEndian>()?;
        }

        let hashed_chunk_list = FileChunkReference::from_reader(reader, 64, 32)?;
        let transaction_log = FileChunkReference::from_reader(reader, 64, 32)?;
        let file_node_list_root = FileChunkReference::from_reader(reader, 64, 32)?;
        let free_chunk_list = FileChunkReference::from_reader(reader, 64, 32)?;
        let expected_file_length = reader.read_u64::<LittleEndian>()?;
        let free_space_in_free_chunk_list = reader.read_u64::<LittleEndian>()?;
        let file_version = Guid::from_reader(reader)?;
        let file_version_generation = reader.read_u64::<LittleEndian>()?;

        // Ignore rest of header fields.

        Ok(OneNoteFileHeader { 
            file_type, 
            file_guid, 
            transactions_in_log, 
            ancestor_guid, 
            hashed_chunk_list,
            transaction_log, 
            file_node_list_root, 
            free_chunk_list, 
            expected_file_length, 
            free_space_in_free_chunk_list, 
            file_version, 
            file_version_generation 
        })
    }
}