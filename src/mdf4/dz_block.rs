use std::io::Read;

use crate::utils;

use super::block::{Block, DataBlock};
use super::block_header::*;
use super::mdf4_enums::ZipType;

use flate2::read::ZlibDecoder;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Dzblock {
    header: BlockHeader,

    dz_org_block_type: String,

    dz_zip_type: u8,
    dz_reserved: u8,

    dz_zip_parameter: u8,

    dz_org_data_length: u64,

    dz_data_length: u64,

    dz_data: Vec<u8>,
}

impl DataBlock for Dzblock {
    fn data_array(&self, _stream: &[u8], _little_endian: bool) -> Vec<u8> {
        let mut zlib_decoder = ZlibDecoder::new(&self.dz_data[..]);
        let mut decompressed_data = vec![0u8; self.dz_org_data_length as usize];
 
        let decompress_result = zlib_decoder.read_exact(&mut decompressed_data);
        match decompress_result {
            Ok(_) => {},
            Err(e) => {
                panic!("{:?}", e);
            }
        }

        decompressed_data
    }
}

impl Block for Dzblock {
    fn read(bytes: &[u8], pos: usize, little_endian: bool) -> Result<(usize, Self), String> {
        let mut pos = pos;
        let (new_pos, header) = BlockHeader::read(bytes, pos, little_endian)?;
        pos = new_pos;

        if &header.id != b"##DZ" {
            return Err(format!("Invalid DZ block identifier: {:?}", header.id));
        }

        let mut block_type = [0u8; 2];
        block_type.copy_from_slice(&bytes[pos..pos + 2]);
        let dz_org_block_type = String::from_utf8_lossy(&block_type).to_string();
        pos += 2;

        let dz_zip_type = bytes[pos];
        pos += 1;

        let dz_zip_parameter = bytes[pos];
        pos += 1;

        let mut org_length_bytes = [0u8; 8];
        org_length_bytes.copy_from_slice(&bytes[pos..pos + 8]);
        let dz_org_data_length = if little_endian {
            u64::from_le_bytes(org_length_bytes)
        } else {
            u64::from_be_bytes(org_length_bytes)
        };
        pos += 8;

        let mut data_length_bytes = [0u8; 8];
        data_length_bytes.copy_from_slice(&bytes[pos..pos + 8]);
        let dz_data_length = if little_endian {
            u64::from_le_bytes(data_length_bytes)
        } else {
            u64::from_be_bytes(data_length_bytes)
        };
        pos += 8;

        let mut dz_data = Vec::new();
        dz_data.extend_from_slice(&bytes[pos..pos + dz_data_length as usize]);
        pos += dz_data_length as usize;

        Ok((pos, Self {
            header,
            dz_org_block_type,
            dz_zip_type,
            dz_reserved: 0,
            dz_zip_parameter,
            dz_org_data_length,
            dz_data_length,
            dz_data,
        }))
    }

    fn read_at(bytes: &[u8], pos: usize, little_endian: bool) -> Result<(usize, Self), String> {
        Self::read(bytes, pos, little_endian)
    }

    fn byte_len(&self) -> usize {
        24 + 2 + 1 + 1 + 8 + 8 + self.dz_data.len()
    }
}

impl Dzblock {
    pub fn new() -> Self {
        Self {
            header: BlockHeader::new(b"##DZ"),
            dz_org_block_type: String::new(),
            dz_zip_type: 0,
            dz_reserved: 0,
            dz_zip_parameter: 0,
            dz_org_data_length: 0,
            dz_data_length: 0,
            dz_data: Vec::new(),
        }
    }

    pub fn data(&self) -> &[u8] {
        &self.dz_data
    }
}
