use std::mem;

use super::block::Block;
use crate::utils;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Idblock {
    #[allow(dead_code)]
    id_file: [u8; 8],
    #[allow(dead_code)]
    id_vers: [u8; 8],
    #[allow(dead_code)]
    id_prog: [u8; 8],
    #[allow(dead_code)]
    id_reserved1: [u8; 4],
    #[allow(dead_code)]
    id_ver: u16,
    #[allow(dead_code)]
    id_reserved2: [u8; 34],
}

impl Idblock {
    // Helper method to get version as string
    pub fn version_string(&self) -> String {
        String::from_utf8_lossy(&self.id_vers).trim().to_string()
    }

    // Helper method to get version as float
    pub fn version_number(&self) -> f32 {
        self.version_string().trim().parse().unwrap_or(4.0)
    }

    // Helper method to get program identifier
    pub fn program_id(&self) -> String {
        String::from_utf8_lossy(&self.id_prog).trim().to_string()
    }
}

impl Block for Idblock {
    fn new() -> Self {
        Self {
            id_file: *b"MDF     ",
            id_vers: *b"4.11    ",
            id_prog: *b"        ",
            id_reserved1: [0; 4],
            id_ver: 411,
            id_reserved2: [0; 34],
        }
    }

    fn default() -> Self {
        Self::new()
    }

    fn read(stream: &[u8], _position: usize, _little_endian: bool) -> (usize, Self) {
        let mut pos = _position;

        // Read fixed-size arrays
        let mut id_file: [u8; 8] = [0; 8];
        id_file.copy_from_slice(&stream[pos..pos + 8]);
        pos += 8;

        let mut id_vers: [u8; 8] = [0; 8];
        id_vers.copy_from_slice(&stream[pos..pos + 8]);
        pos += 8;

        let mut id_prog: [u8; 8] = [0; 8];
        id_prog.copy_from_slice(&stream[pos..pos + 8]);
        pos += 8;

        let mut id_reserved1: [u8; 4] = [0; 4];
        id_reserved1.copy_from_slice(&stream[pos..pos + 4]);
        pos += 4;

        let id_ver: u16 = utils::read(stream, _little_endian, &mut pos);

        let mut id_reserved2: [u8; 34] = [0; 34];
        id_reserved2.copy_from_slice(&stream[pos..pos + 34]);
        pos += 34;

        // Validate the ID block
        if !utils::eq(&id_file, b"MDF     ") {
            panic!("Invalid MDF4 file identifier");
        }

        (
            pos,
            Self {
                id_file,
                id_vers,
                id_prog,
                id_reserved1,
                id_ver,
                id_reserved2,
            },
        )
    }

    fn byte_len(&self) -> usize {
        mem::size_of_val(&self.id_file)
            + mem::size_of_val(&self.id_vers)
            + mem::size_of_val(&self.id_prog)
            + mem::size_of_val(&self.id_reserved1)
            + mem::size_of_val(&self.id_ver)
            + mem::size_of_val(&self.id_reserved2)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Test data for v4.10
    static RAW_410: [u8; 64] = [
        0x4D, 0x44, 0x46, 0x20, 0x20, 0x20, 0x20, 0x20, 0x34, 0x2E, 0x31, 0x30, 0x20, 0x20, 0x20,
        0x20, 0x54, 0x47, 0x54, 0x20, 0x31, 0x35, 0x2E, 0x30, 0x00, 0x00, 0x00, 0x00, 0x9A, 0x01,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00,
    ];

    // Test data for v4.11
    static RAW_411: [u8; 64] = [
        0x4D, 0x44, 0x46, 0x20, 0x20, 0x20, 0x20, 0x20, 0x34, 0x2E, 0x31, 0x31, 0x20, 0x20, 0x20,
        0x20, 0x48, 0x61, 0x70, 0x70, 0x79, 0x62, 0x6F, 0x78, 0x00, 0x00, 0x00, 0x00, 0x9B, 0x01,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00,
    ];

    #[test]
    fn read_v410() {
        let (pos, id_result) = Idblock::read(&RAW_410, 0, true);

        assert_eq!(64, pos);
        assert!(utils::eq("MDF     ".as_bytes(), &id_result.id_file));
        assert!(utils::eq("4.10    ".as_bytes(), &id_result.id_vers));
        assert!(utils::eq("TGT 15.0".as_bytes(), &id_result.id_prog));
        assert!(utils::eq(&[0_u8; 4], &id_result.id_reserved1));
        assert_eq!(410, id_result.id_ver);
        assert!(utils::eq(&[0_u8; 34], &id_result.id_reserved2));
        assert_eq!("4.10", id_result.version_string());
        assert_eq!(4.10, id_result.version_number());
        assert_eq!("TGT 15.0", id_result.program_id());
    }

    #[test]
    fn read_v411() {
        let (pos, id_result) = Idblock::read(&RAW_411, 0, true);

        assert_eq!(64, pos);
        assert!(utils::eq("MDF     ".as_bytes(), &id_result.id_file));
        assert!(utils::eq("4.11    ".as_bytes(), &id_result.id_vers));
        assert!(utils::eq("Happybox".as_bytes(), &id_result.id_prog));
        assert!(utils::eq(&[0_u8; 4], &id_result.id_reserved1));
        assert_eq!(411, id_result.id_ver);
        assert!(utils::eq(&[0_u8; 34], &id_result.id_reserved2));
        assert_eq!("4.11", id_result.version_string());
        assert_eq!(4.11, id_result.version_number());
        assert_eq!("Happybox", id_result.program_id());
    }

    #[test]
    fn byte_len() {
        let (_pos, id_result) = Idblock::read(&RAW_410, 0, true);
        assert_eq!(64, id_result.byte_len());
    }
}
