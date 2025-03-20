use std::mem;

use super::block::Block;
use crate::utils;
use crate::mdf4::block_header::BlockHeader;

#[derive(Debug)]
pub struct Idblock {
    header: BlockHeader,
    id_file_identifier: String,
    id_version: u32,
    id_program_identifier: String,
    id_reserved: [u8; 4],
    id_unfinalized_standard_flags: u16,
    id_unfinalized_custom_flags: u16,
}

impl Block for Idblock {
    fn read(bytes: &[u8], pos: usize, little_endian: bool) -> Result<(usize, Self), String> {
        let mut pos = pos;
        let (new_pos, header) = BlockHeader::read(bytes, pos, little_endian)?;
        pos = new_pos;

        if &header.id != b"##ID" {
            return Err(format!("Invalid ID block identifier: {:?}", header.id));
        }

        let mut file_id_len = [0u8; 4];
        file_id_len.copy_from_slice(&bytes[pos..pos + 4]);
        let file_id_len = if little_endian {
            u32::from_le_bytes(file_id_len)
        } else {
            u32::from_be_bytes(file_id_len)
        } as usize;
        pos += 4;

        let id_file_identifier = String::from_utf8_lossy(&bytes[pos..pos + file_id_len]).to_string();
        pos += file_id_len;

        let mut version_bytes = [0u8; 4];
        version_bytes.copy_from_slice(&bytes[pos..pos + 4]);
        let id_version = if little_endian {
            u32::from_le_bytes(version_bytes)
        } else {
            u32::from_be_bytes(version_bytes)
        };
        pos += 4;

        let mut program_id_len = [0u8; 4];
        program_id_len.copy_from_slice(&bytes[pos..pos + 4]);
        let program_id_len = if little_endian {
            u32::from_le_bytes(program_id_len)
        } else {
            u32::from_be_bytes(program_id_len)
        } as usize;
        pos += 4;

        let id_program_identifier = String::from_utf8_lossy(&bytes[pos..pos + program_id_len]).to_string();
        pos += program_id_len;

        let mut id_reserved = [0u8; 4];
        id_reserved.copy_from_slice(&bytes[pos..pos + 4]);
        pos += 4;

        let mut standard_flags_bytes = [0u8; 2];
        standard_flags_bytes.copy_from_slice(&bytes[pos..pos + 2]);
        let id_unfinalized_standard_flags = if little_endian {
            u16::from_le_bytes(standard_flags_bytes)
        } else {
            u16::from_be_bytes(standard_flags_bytes)
        };
        pos += 2;

        let mut custom_flags_bytes = [0u8; 2];
        custom_flags_bytes.copy_from_slice(&bytes[pos..pos + 2]);
        let id_unfinalized_custom_flags = if little_endian {
            u16::from_le_bytes(custom_flags_bytes)
        } else {
            u16::from_be_bytes(custom_flags_bytes)
        };
        pos += 2;

        Ok((pos, Self {
            header,
            id_file_identifier,
            id_version,
            id_program_identifier,
            id_reserved,
            id_unfinalized_standard_flags,
            id_unfinalized_custom_flags,
        }))
    }

    fn read_at(bytes: &[u8], pos: usize, little_endian: bool) -> Result<(usize, Self), String> {
        Self::read(bytes, pos, little_endian)
    }

    fn byte_len(&self) -> usize {
        24 + 4 + self.id_file_identifier.len() + 4 + 4 + self.id_program_identifier.len() + 4 + 2 + 2
    }
}

impl Idblock {
    pub fn new() -> Self {
        Self {
            header: BlockHeader::new(b"##ID"),
            id_file_identifier: String::new(),
            id_version: 0,
            id_program_identifier: String::new(),
            id_reserved: [0; 4],
            id_unfinalized_standard_flags: 0,
            id_unfinalized_custom_flags: 0,
        }
    }

    pub fn version(&self) -> u32 {
        self.id_version
    }
}

#[cfg(test)]

mod tests {
    use crate::{
        mdf4::{block::Block, id_block::Idblock},
        utils,
    };

    static RAW: [u8; 64] = [
        0x4D, 0x44, 0x46, 0x20, 0x20, 0x20, 0x20, 0x20, 0x34, 0x2E, 0x31, 0x30, 0x20, 0x20, 0x20,
        0x20, 0x54, 0x47, 0x54, 0x20, 0x31, 0x35, 0x2E, 0x30, 0x00, 0x00, 0x00, 0x00, 0x9A, 0x01,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00,
    ];

    #[test]
    fn read() {
        let (pos, id_result) = Idblock::read(&RAW, 0, true);

        assert_eq!(64, pos);
        assert!(utils::eq("MDF     ".as_bytes(), &id_result.id_file_identifier.as_bytes()));
        assert!(utils::eq("4.10    ".as_bytes(), &id_result.id_program_identifier.as_bytes()));
        assert!(utils::eq("TGT 15.0".as_bytes(), &id_result.id_program_identifier.as_bytes()));
        assert!(utils::eq(&[0_u8; 4], &id_result.id_reserved));
        assert_eq!(410, id_result.id_version);
        assert!(utils::eq(&[0_u8; 2], &id_result.id_unfinalized_standard_flags.to_le_bytes()));
        assert!(utils::eq(&[0_u8; 2], &id_result.id_unfinalized_custom_flags.to_le_bytes()));
    }

    #[test]
    fn byte_len() {
        let (_pos, id_result) = Idblock::read(&RAW, 0, true);

        assert_eq!(64, id_result.byte_len());
    }
}
