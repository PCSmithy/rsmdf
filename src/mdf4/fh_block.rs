use std::mem;

use super::block::Block;
use super::block_header::*;
use super::mdf4_file::link_extract;
use crate::utils;

#[derive(Debug, Clone, PartialEq)]
pub struct Fhblock {
    header: BlockHeader,
    links: Vec<u64>,
}

impl Fhblock {
    pub fn comment_addr(&self) -> Option<u64> {
        // The first link in an FH block points to the comment (TX block)
        self.links.first().copied()
    }
}

impl Block for Fhblock {
    fn new() -> Self {
        Self {
            header: BlockHeader::create("##FH", 56, 2),
            links: vec![0, 0],
        }
    }

    fn default() -> Self {
        Self::new()
    }

    fn read(stream: &[u8], position: usize, little_endian: bool) -> (usize, Self) {
        let (pos, header) = BlockHeader::read(stream, position, little_endian);

        if !utils::eq(&header.id, "##FH".as_bytes()) {
            panic!("Error type incorrect");
        }

        let mut links = Vec::new();
        let mut current_pos = pos;

        // Read all links
        for _ in 0..header.link_count {
            let link: u64 = utils::read(stream, little_endian, &mut current_pos);
            links.push(link);
        }

        (current_pos, Self { header, links })
    }

    fn byte_len(&self) -> usize {
        24 + (self.links.len() * 8)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Test data for an FH block with two links
    static RAW: [u8; 56] = [
        // Block Header
        0x23, 0x23, 0x46, 0x48, // "##FH"
        0x00, 0x00, 0x00, 0x00, // Reserved
        0x38, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // Length = 56
        0x02, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // Link count = 2
        // Links
        0xE0, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // Link 1 = 224 (TX block)
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // Link 2 = 0 (Next FH block)
        // Additional data (if any)
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00,
    ];

    #[test]
    fn read() {
        let (pos, fh_block) = Fhblock::read(&RAW, 0, true);

        assert_eq!(40, pos);
        assert!(utils::eq(&fh_block.header.id, "##FH".as_bytes()));
        assert_eq!(56, fh_block.header.length);
        assert_eq!(2, fh_block.header.link_count);
        assert_eq!(2, fh_block.links.len());
        assert_eq!(224, fh_block.links[0]); // TX block address
        assert_eq!(0, fh_block.links[1]); // Next FH block address
    }

    #[test]
    fn byte_len() {
        let (_, fh_block) = Fhblock::read(&RAW, 0, true);
        assert_eq!(40, fh_block.byte_len());
    }
}
