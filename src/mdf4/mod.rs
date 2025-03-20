mod at_block;
mod block;
mod block_header;
mod ca_block;
mod cc_block;
mod cg_block;
mod ch_block;
mod cn_block;
mod data_block;
mod dg_block;
mod dl_block;
mod dt_block;
mod dz_block;
mod ev_block;
mod fh_block;
mod hd_block;
mod hl_block;
mod id_block;
mod md_block;
mod mdf4_enums;
pub mod mdf4_file;
mod sd_block;
mod si_block;
mod sr_block;
mod tx_block;
mod utils;

use std::fs::File;
use std::io::Read;
use std::path::Path;

use block::Block;
use block_header::BlockHeader;
use channel::Channel;
use md_block::Mdblock;
use crate::mdf4::block::Block as MDF4Block;
use crate::mdf4::block_header::BlockHeader as MDF4BlockHeader;
use crate::mdf4::dg_block::Dgblock;
use crate::mdf4::hd_block::Hdblock;
use crate::mdf4::id_block::Idblock;

#[derive(Debug)]
pub struct MDF4 {
    id_block: Idblock,
    hd_block: Hdblock,
    dg_blocks: Vec<Dgblock>,
}

impl MDF4 {
    pub fn new() -> Self {
        Self {
            id_block: Idblock::new(),
            hd_block: Hdblock::new(),
            dg_blocks: Vec::new(),
        }
    }

    pub fn read(bytes: &[u8], pos: usize, little_endian: bool) -> Result<(usize, Self), String> {
        let mut pos = pos;
        let (new_pos, id_block) = Idblock::read(bytes, pos, little_endian)?;
        pos = new_pos;

        let (new_pos, hd_block) = Hdblock::read(bytes, pos, little_endian)?;
        pos = new_pos;

        let mut dg_blocks = Vec::new();
        let mut current_dg_block = hd_block.hd_first_dg_block;

        while current_dg_block != 0 {
            let (new_pos, dg_block) = Dgblock::read(bytes, current_dg_block as usize, little_endian)?;
            current_dg_block = dg_block.dg_next_dg_block;
            dg_blocks.push(dg_block);
        }

        Ok((pos, Self {
            id_block,
            hd_block,
            dg_blocks,
        }))
    }

    pub fn id_block(&self) -> &Idblock {
        &self.id_block
    }

    pub fn hd_block(&self) -> &Hdblock {
        &self.hd_block
    }

    pub fn dg_blocks(&self) -> &[Dgblock] {
        &self.dg_blocks
    }

    pub fn read_file<P: AsRef<Path>>(&mut self, path: P) -> Result<(), String> {
        let mut file = File::open(path).map_err(|e| e.to_string())?;
        let mut bytes = Vec::new();
        file.read_to_end(&mut bytes).map_err(|e| e.to_string())?;

        self.read_bytes(&bytes)
    }

    pub fn read_bytes(&mut self, bytes: &[u8]) -> Result<(), String> {
        let mut pos = 0;
        
        // Read file header
        let (new_pos, _header) = BlockHeader::read(bytes, pos, true)?;
        pos = new_pos;

        // Read metadata blocks
        while pos < bytes.len() {
            let mut channel = Channel::new(String::new(), String::new(), String::new());
            pos = channel.read_metadata(bytes, pos)?;
            self.channels.push(channel);
        }

        Ok(())
    }

    pub fn channels(&self) -> &[Channel] {
        &self.channels
    }

    pub fn channel_by_name(&self, name: &str) -> Option<&Channel> {
        self.channels.iter().find(|c| c.name() == name)
    }
}
