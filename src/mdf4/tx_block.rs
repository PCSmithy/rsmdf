use super::block::Block;
use super::block_header::*;
use super::utils as mdf4_utils;
use crate::utils;
use std::error::Error;
use std::fmt;

#[derive(Debug)]
pub struct TxBlockError {
    message: String,
}

impl fmt::Display for TxBlockError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl Error for TxBlockError {}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Txblock {
    header: BlockHeader,
    tx_data: String,
}

impl Txblock {
    pub fn text(&self) -> String {
        self.clone().tx_data
    }
}

impl Block for Txblock {
    fn new() -> Self {
        Self {
            header: BlockHeader::create("##TX", 24, 0),
            tx_data: String::new(),
        }
    }
    fn default() -> Self {
        Self {
            header: BlockHeader::create("##TX", 24, 0),
            tx_data: String::new(),
        }
    }
    fn read(stream: &[u8], position: usize, little_endian: bool) -> (usize, Self) {
        match Self::try_read(stream, position, little_endian) {
            Ok((pos, block)) => (pos, block),
            Err(e) => panic!("{}", e), // maintain backward compatibility for now
        }
    }

    fn byte_len(&self) -> usize {
        self.header.byte_len() + self.tx_data.len() + 1 // add 1 for the trailing null on a c string
    }
}

impl Txblock {
    pub fn try_read(
        stream: &[u8],
        position: usize,
        little_endian: bool,
    ) -> Result<(usize, Self), TxBlockError> {
        let (pos, header) = BlockHeader::read(stream, position, little_endian);

        if !utils::eq(&header.id, "##TX".as_bytes()) {
            return Err(TxBlockError {
                message: format!(
                    "Expected TX block but found: {:?} at {}",
                    &header.id, position
                ),
            });
        }

        let length = header.length as usize - header.byte_len();
        let tx_data = mdf4_utils::str_from_u8(&stream[pos..(pos + length)]);

        Ok((pos + length, Self { header, tx_data }))
    }
}

#[cfg(test)]
mod tests {
    use crate::mdf4::{block::Block, tx_block::Txblock};

    static RAW: [u8; 40] = [
        0x23, 0x23, 0x54, 0x58, 0x00, 0x00, 0x00, 0x00, 0x21, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x45, 0x6E, 0x67, 0x69, 0x6E, 0x65,
        0x5F, 0x31, 0x00, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40,
    ];

    #[test]
    fn read() {
        let (pos, tx) = Txblock::read(&RAW, 0, true);

        assert_eq!(33, pos);
        //println!("{}", tx.tx_data);
        assert!(tx.tx_data.eq("Engine_1"))
    }

    #[test]
    fn byte_len() {
        let (pos, tx) = Txblock::read(&RAW, 0, true);

        assert_eq!(33, pos);
        assert_eq!(33, tx.byte_len());
    }
}
