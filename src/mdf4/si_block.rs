use std::mem;

use crate::utils;

use super::block::Block;
use super::block_header::*;
use super::mdf4_enums::{BusType, SourceType};
use super::mdf4_file::link_extract;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Siblock {
    header: BlockHeader,
    pub si_tx_name: u64,
    pub si_tx_path: u64,
    pub si_md_comment: u64,
    pub si_type: SourceType,
    pub si_bus_type: BusType,
    pub si_flags: u8,
    pub si_reserved: [u8; 5],
}
impl Block for Siblock {
    fn new() -> Self {
        Siblock {
            header: BlockHeader::create("##SI", 50, 0),
            si_tx_name: 0_u64,
            si_tx_path: 0_u64,
            si_md_comment: 0_u64,
            si_type: SourceType::Bus,
            si_bus_type: BusType::Can,
            si_flags: 0_u8,
            si_reserved: [0_u8; 5],
        }
    }
    fn default() -> Self {
        Siblock {
            header: BlockHeader::create("##SI", 50, 0),
            si_tx_name: 0_u64,
            si_tx_path: 0_u64,
            si_md_comment: 0_u64,
            si_type: SourceType::Bus,
            si_bus_type: BusType::Can,
            si_flags: 0_u8,
            si_reserved: [0_u8; 5],
        }
    }
    fn read(stream: &[u8], position: usize, little_endian: bool) -> (usize, Self) {
        let (pos, header) = BlockHeader::read(stream, position, little_endian);

        if !utils::eq(&header.id, "##SI".as_bytes()) {
            panic!("Error SIBLOCK");
        }

        let (mut pos, mut address) = link_extract(stream, pos, little_endian, header.link_count);

        let si_tx_name = address.remove(0);
        let si_tx_path = address.remove(0);
        let si_md_comment = address.remove(0);

        let si_type = SourceType::new(utils::read(stream, little_endian, &mut pos));
        let si_bus_type = BusType::new(utils::read(stream, little_endian, &mut pos));
        let si_flags = utils::read(stream, little_endian, &mut pos);

        let si_reserved = utils::read(stream, little_endian, &mut pos);

        (
            pos,
            Siblock {
                header,
                si_tx_name,
                si_tx_path,
                si_md_comment,
                si_type,
                si_bus_type,
                si_flags,
                si_reserved,
            },
        )
    }

    fn byte_len(&self) -> usize {
        self.header.byte_len()
            + mem::size_of_val(&self.si_tx_name)
            + mem::size_of_val(&self.si_tx_path)
            + mem::size_of_val(&self.si_md_comment)
            + mem::size_of_val(&self.si_type)
            + mem::size_of_val(&self.si_bus_type)
            + mem::size_of_val(&self.si_flags)
            + mem::size_of_val(&self.si_reserved)
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        mdf4::{
            block::Block,
            mdf4_enums::{BusType, SourceType},
            si_block::Siblock,
        },
        utils,
    };

    static RAW: [u8; 56] = [
        0x23, 0x23, 0x53, 0x49, 0x00, 0x00, 0x00, 0x00, 0x38, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x03, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xF0, 0x44, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x10, 0x45, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x01, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    ];

    #[test]
    fn read() {
        let (pos, si) = Siblock::read(&RAW, 0, true);

        assert_eq!(pos, 56);
        assert_eq!(17648, si.si_tx_name);
        assert_eq!(17680, si.si_tx_path);
        assert_eq!(0, si.si_md_comment);
        assert_eq!(SourceType::Ecu, si.si_type);
        assert_eq!(BusType::Other, si.si_bus_type);
        assert_eq!(0, si.si_flags);
        assert!(utils::eq(&si.si_reserved, &[0_u8; 5]));
    }

    #[test]
    fn test_bus_type_parsing() {
        // Create a template SI block with default values
        let mut raw = RAW.clone();

        // The bus type byte is at offset 49 in the raw data
        let bus_type_offset = 49;

        // Test all bus types
        let test_cases = vec![
            (0, BusType::None),
            (1, BusType::Other),
            (2, BusType::Can),
            (3, BusType::Lin),
            (4, BusType::Most),
            (5, BusType::FlexRay),
            (6, BusType::KLine),
            (7, BusType::Ethernet),
            (8, BusType::Usb),
        ];

        for (value, expected_type) in test_cases {
            // Modify the bus type byte in the raw data
            raw[bus_type_offset] = value;

            // Parse the modified raw data
            let (_, si) = Siblock::read(&raw, 0, true);

            // Verify the parsed bus type matches the expected type
            assert_eq!(
                si.si_bus_type, expected_type,
                "Failed to parse bus type value {} as {:?}",
                value, expected_type
            );
        }
    }

    #[test]
    fn byte_len() {
        let (pos, si) = Siblock::read(&RAW, 0, true);

        assert_eq!(pos, si.byte_len());
    }
}
