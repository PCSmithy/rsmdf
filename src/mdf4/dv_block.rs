use crate::mdf4::block::Block;
use crate::mdf4::block_header::BlockHeader;

#[derive(Debug)]
pub struct Dvblock {
    header: BlockHeader,
    dv_next_dv_block: u64,
    dv_data_block: u64,
    dv_parent_dg_block: u64,
    dv_data_type: u8,
    dv_reserved: [u8; 3],
    dv_data: Vec<u8>,
}

impl Block for Dvblock {
    fn read(bytes: &[u8], pos: usize, little_endian: bool) -> Result<(usize, Self), String> {
        let mut pos = pos;
        let (new_pos, header) = BlockHeader::read(bytes, pos, little_endian)?;
        pos = new_pos;

        if &header.id != b"##DV" {
            return Err(format!("Invalid DV block identifier: {:?}", header.id));
        }

        let mut next_dv_bytes = [0u8; 8];
        next_dv_bytes.copy_from_slice(&bytes[pos..pos + 8]);
        let dv_next_dv_block = if little_endian {
            u64::from_le_bytes(next_dv_bytes)
        } else {
            u64::from_be_bytes(next_dv_bytes)
        };
        pos += 8;

        let mut data_block_bytes = [0u8; 8];
        data_block_bytes.copy_from_slice(&bytes[pos..pos + 8]);
        let dv_data_block = if little_endian {
            u64::from_le_bytes(data_block_bytes)
        } else {
            u64::from_be_bytes(data_block_bytes)
        };
        pos += 8;

        let mut parent_dg_bytes = [0u8; 8];
        parent_dg_bytes.copy_from_slice(&bytes[pos..pos + 8]);
        let dv_parent_dg_block = if little_endian {
            u64::from_le_bytes(parent_dg_bytes)
        } else {
            u64::from_be_bytes(parent_dg_bytes)
        };
        pos += 8;

        let dv_data_type = bytes[pos];
        pos += 1;

        let mut dv_reserved = [0u8; 3];
        dv_reserved.copy_from_slice(&bytes[pos..pos + 3]);
        pos += 3;

        let mut data_len = [0u8; 8];
        data_len.copy_from_slice(&bytes[pos..pos + 8]);
        let data_len = if little_endian {
            u64::from_le_bytes(data_len)
        } else {
            u64::from_be_bytes(data_len)
        } as usize;
        pos += 8;

        let mut dv_data = Vec::new();
        dv_data.extend_from_slice(&bytes[pos..pos + data_len]);
        pos += data_len;

        Ok((pos, Self {
            header,
            dv_next_dv_block,
            dv_data_block,
            dv_parent_dg_block,
            dv_data_type,
            dv_reserved,
            dv_data,
        }))
    }

    fn read_at(bytes: &[u8], pos: usize, little_endian: bool) -> Result<(usize, Self), String> {
        Self::read(bytes, pos, little_endian)
    }

    fn byte_len(&self) -> usize {
        24 + 8 + 8 + 8 + 1 + 3 + 8 + self.dv_data.len()
    }
}

impl Dvblock {
    pub fn new() -> Self {
        Self {
            header: BlockHeader::new(b"##DV"),
            dv_next_dv_block: 0,
            dv_data_block: 0,
            dv_parent_dg_block: 0,
            dv_data_type: 0,
            dv_reserved: [0; 3],
            dv_data: Vec::new(),
        }
    }

    pub fn data(&self) -> &[u8] {
        &self.dv_data
    }
} 