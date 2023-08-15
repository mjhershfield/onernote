use std::io::Read;
use std::io::Error;
use std::io::ErrorKind;
use std::io::Seek;
use std::io::SeekFrom;

use byteorder::LittleEndian;
use byteorder::ReadBytesExt;
use packed_struct::prelude::*;
use packed_struct::EnumCatchAll::*;

use super::filechunkreference::FileChunkReference;

// TODO: REIMPLEMENT FILE NODE HEADER PARSING WITHOUT PACKED STRUCT

#[derive(PackedStruct)]
#[packed_struct(endian="msb", bit_numbering="lsb0", size_bytes="4")]
pub struct FileNodeHeader {
    #[packed_field(bits="0:9", ty="enum")]
    id: EnumCatchAll<FileType>,
    #[packed_field(bits="10:22")]
    size: Integer<u16, packed_bits::Bits::<13>>,
    #[packed_field(bits="23:24", ty="enum")]
    stp_format: StpFormat,
    #[packed_field(bits="25:26", ty="enum")]
    cb_format: CbFormat,
    #[packed_field(bits="27:30", ty="enum")]
    base_type: EnumCatchAll<BaseType>,
    #[packed_field(bits="31")]
    reserved: Integer<u8, packed_bits::Bits::<1>>
}

#[derive(PrimitiveEnum, Clone, Copy, PartialEq, Debug)]
pub enum StpFormat {
    Uncompressed8,
    Uncompressed4,
    Compressed2,
    Compressed4
}

#[derive(PrimitiveEnum, Clone, Copy, PartialEq, Debug)]
pub enum CbFormat {
    Uncompressed4,
    Uncompressed8,
    Compressed1,
    Compressed2
}

#[derive(PrimitiveEnum_u8, Clone, Copy, PartialEq, Debug)]
pub enum BaseType {
    NoReference,
    DataReference,
    FileNodeReference,
}

#[derive(PrimitiveEnum_u16, Clone, Copy, PartialEq, Debug)]
pub enum FileType {
    ObjectSpaceManifestRoot = 0x04,
    ObjectSpaceManifestListReference = 0x08,
    ObjectSpaceManifestListStart = 0x0C,
    RevisionManifestListReference = 0x10,
    RevisionManifestListStart = 0x14,
    RevisionManifestStart4 = 0x1B,
    RevisionManifestEnd = 0x1C,
    RevisionManifestStart6 = 0x1E,
    RevisionManifestStart7 = 0x1F,
    GlobalIdTableStart = 0x21,
    GlobalIdTableStart2 = 0x22,
    GlobalIdTableEntry = 0x24,
    GlobalIdTableEntry2 = 0x25,
    GlobalIdTableEntry3 = 0x26,
    GlobalIdTableEnd = 0x28,
    ObjectDeclarationWithRefCount = 0x2D,
    ObjectDeclarationWithRefCount2 = 0x2E,
    ObjectRevisionWithRefCount = 0x41,
    ObjectRevisionWithRefCount2 = 0x42,
    RootObjectReference2 = 0x59,
    RootObjectReference3 = 0x5A,
    RevisionRoleDeclaration = 0x5C,
    RevisionRoleAndContextDeclaration = 0x5D,
    ObjectDeclarationFileData3RefCount = 0x72,
    ObjectDeclarationFileData3LargeRefCount = 0x73,
    ObjectDataEncryptionKeyV2 = 0x7C,
    ObjectInfoDependencyOverrides = 0x84,
    DataSignatureGroupDefinition = 0x8C,
    FileDataStoreListReference = 0x90,
    FileDataStoreObjectReference = 0x94,
    ObjectDeclaration2RefCount = 0xA4,
    ObjectDeclaration2LargeRefCount = 0xA5,
    ObjectGroupListReference = 0xB0,
    ObjectGroupStart = 0xB4,
    ObjectGroupEnd = 0xB8,
    HashedChunkDescriptor2 = 0xC2,
    ReadOnlyObjectDeclaration2RefCount = 0xC4,
    ReadOnlyObjectDeclaration2LargeRefCount = 0xC5,
    ChunkTerminator = 0xFF
}

// TODO: WHAT IF WE DID SOME KIND OF BOX<OBJECT> OR AN OBJECT ENUM? WOULD THAT BE A WASTE OF SPACE?
#[derive(Debug)]
pub struct FileNode {
    pub file_type: FileType,
    pub size: u16,
    pub file_chunk_ref: FileChunkReference,
    pub base_type: BaseType
}

// TODO: DO WE WANT TO BE ABLE TO READ FILE NODES FROM ARBITRARY FCRS OR JUST RELY ON CURRENT READER POSITION? (fromfilechunk?)
impl FileNode {
    pub fn from_reader<T: Read + Seek>(reader: &mut T) -> Result<Self, std::io::Error> where Self: Sized {
        let start_of_file_node = reader.stream_position()?;
        
        // Unpack file node header
        let mut header_buffer: [u8; 4] = [0; 4];
        reader.read_exact(&mut header_buffer)?;
        header_buffer.reverse();
        let header = FileNodeHeader::unpack(&header_buffer).unwrap();

        // Parse file type from file node id field
        let file_type: FileType;
        match header.id {
            Enum(e) => file_type = e,
            CatchAll(_) => return Err(Error::new(ErrorKind::InvalidData, "Invalid FileNodeHeader id field"))
        }

        let base_type: BaseType;
        let fcr_start: u64;
        let fcr_len: u64;

        // Parse base type from node id field
        match header.base_type {
            Enum(e) => base_type = e,
            CatchAll(_) => return Err(Error::new(ErrorKind::InvalidData, "Invalid base type"))
        }

        // Depending on base type, stp format, and cb format, create file chunk reference for node body
        match base_type {
            BaseType::DataReference | BaseType::FileNodeReference => {
                match header.stp_format {
                    StpFormat::Uncompressed8 => fcr_start = reader.read_u64::<LittleEndian>()?,
                    StpFormat::Uncompressed4 => fcr_start = reader.read_u32::<LittleEndian>()?.into(),
                    StpFormat::Compressed2 => fcr_start = reader.read_u16::<LittleEndian>()? as u64 * 8,
                    StpFormat::Compressed4 => fcr_start = reader.read_u32::<LittleEndian>()? as u64 * 8
                }
                match header.cb_format {
                    CbFormat::Uncompressed8 => fcr_len = reader.read_u64::<LittleEndian>()?,
                    CbFormat::Uncompressed4 => fcr_len = reader.read_u32::<LittleEndian>()?.into(),
                    CbFormat::Compressed2 => fcr_len = reader.read_u16::<LittleEndian>()? as u64 * 8,
                    CbFormat::Compressed1 => fcr_len = reader.read_u8()? as u64 * 8
                }
            },
            BaseType::NoReference => {
                fcr_start = reader.stream_position()?;
                // TODO: GET LEN OF FCR BLOCKS FOR NON REFERENCE DATATYPES
                fcr_len = 0;
            }
        }

        // Simulate reading through this file node's data
        let node_size: u16 = header.size.into();
        reader.seek(SeekFrom::Start(start_of_file_node + node_size as u64))?;

        Ok(FileNode {
            file_type,
            size: header.size.into(),
            file_chunk_ref: FileChunkReference { start: fcr_start, len: fcr_len },
            base_type
        })
    }
}