use std::mem;

use super::block::Block;
use crate::utils;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BlockHeader {
    pub id: [u8; 4],
    pub reserved: [u8; 4],
    pub length: u64,
    pub link_count: u64,
}

impl Block for BlockHeader {
    fn read(bytes: &[u8], pos: usize, little_endian: bool) -> Result<(usize, Self), String> {
        let mut pos = pos;
        let mut id = [0u8; 4];
        id.copy_from_slice(&bytes[pos..pos + 4]);
        pos += 4;

        let mut reserved = [0u8; 4];
        reserved.copy_from_slice(&bytes[pos..pos + 4]);
        pos += 4;

        let mut length_bytes = [0u8; 8];
        length_bytes.copy_from_slice(&bytes[pos..pos + 8]);
        let length = if little_endian {
            u64::from_le_bytes(length_bytes)
        } else {
            u64::from_be_bytes(length_bytes)
        };
        pos += 8;

        let mut link_count_bytes = [0u8; 8];
        link_count_bytes.copy_from_slice(&bytes[pos..pos + 8]);
        let link_count = if little_endian {
            u64::from_le_bytes(link_count_bytes)
        } else {
            u64::from_be_bytes(link_count_bytes)
        };
        pos += 8;

        Ok((pos, Self {
            id,
            reserved,
            length,
            link_count,
        }))
    }

    fn read_at(bytes: &[u8], pos: usize, little_endian: bool) -> Result<(usize, Self), String> {
        Self::read(bytes, pos, little_endian)
    }

    fn byte_len(&self) -> usize {
        24
    }
}

impl BlockHeader {
    pub fn new(id: &[u8]) -> Self {
        let mut header_id = [0u8; 4];
        header_id.copy_from_slice(id);
        Self {
            id: header_id,
            reserved: [0u8; 4],
            length: 24,
            link_count: 0,
        }
    }

    pub fn default() -> Self {
        Self {
            id: [0u8; 4],
            reserved: [0u8; 4],
            length: 24,
            link_count: 0,
        }
    }
}
