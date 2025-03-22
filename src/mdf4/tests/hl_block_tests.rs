use std::fs::File;
use std::io::Write;
use tempfile::NamedTempFile;

use crate::mdf4::{
    block::Block,
    block_header::BlockHeader,
    data_block::{DataBlockType, Dtblock, Dzblock},
    hl_block::Hlblock,
    mdf4_enums::ZipType,
    utils,
};

fn create_test_hl_block() -> Vec<u8> {
    let mut data = Vec::new();
    
    // Create HL block header (50 bytes)
    let header = BlockHeader::create("##HL", 50, 1); // 1 link for dl_first
    data.extend_from_slice(&header.id);
    data.extend_from_slice(&header.length.to_le_bytes());
    data.extend_from_slice(&header.link_count.to_le_bytes());
    
    // Add link to DL block (8 bytes)
    data.extend_from_slice(&(64_u64).to_le_bytes()); // DL block will be at offset 64
    
    // Add flags (2 bytes)
    data.extend_from_slice(&(0_u16).to_le_bytes());
    
    // Add zip type (1 byte)
    data.extend_from_slice(&(ZipType::None as u8).to_le_bytes());
    
    // Add reserved bytes (5 bytes)
    data.extend_from_slice(&[0_u8; 5]);
    
    // Pad to 50 bytes
    while data.len() < 50 {
        data.push(0);
    }
    
    data
}

fn create_test_dt_block() -> Vec<u8> {
    let mut data = Vec::new();
    
    // Create DT block header (24 bytes)
    let header = BlockHeader::create("##DT", 24, 0);
    data.extend_from_slice(&header.id);
    data.extend_from_slice(&header.length.to_le_bytes());
    data.extend_from_slice(&header.link_count.to_le_bytes());
    
    // Add some test data (4 bytes)
    data.extend_from_slice(&[1, 2, 3, 4]);
    
    // Pad to 24 bytes
    while data.len() < 24 {
        data.push(0);
    }
    
    data
}

fn create_test_dl_block() -> Vec<u8> {
    let mut data = Vec::new();
    
    // Create DL block header (24 bytes)
    let header = BlockHeader::create("##DL", 24, 1); // 1 link for first data block
    data.extend_from_slice(&header.id);
    data.extend_from_slice(&header.length.to_le_bytes());
    data.extend_from_slice(&header.link_count.to_le_bytes());
    
    // Add link to DT block (8 bytes)
    data.extend_from_slice(&(88_u64).to_le_bytes()); // DT block will be at offset 88
    
    // Add flags (2 bytes)
    data.extend_from_slice(&(0_u16).to_le_bytes());
    
    // Add reserved bytes (6 bytes)
    data.extend_from_slice(&[0_u8; 6]);
    
    // Pad to 24 bytes
    while data.len() < 24 {
        data.push(0);
    }
    
    data
}

fn create_test_dz_block() -> Vec<u8> {
    let mut data = Vec::new();
    
    // Create DZ block header (24 bytes)
    let header = BlockHeader::create("##DZ", 24, 0);
    data.extend_from_slice(&header.id);
    data.extend_from_slice(&header.length.to_le_bytes());
    data.extend_from_slice(&header.link_count.to_le_bytes());
    
    // Add original size (8 bytes)
    data.extend_from_slice(&(4_u64).to_le_bytes());
    
    // Add zip type (1 byte)
    data.extend_from_slice(&(ZipType::Deflate as u8).to_le_bytes());
    
    // Add reserved bytes (7 bytes)
    data.extend_from_slice(&[0_u8; 7]);
    
    // Add compressed data (4 bytes)
    data.extend_from_slice(&[1, 2, 3, 4]);
    
    // Pad to 24 bytes
    while data.len() < 24 {
        data.push(0);
    }
    
    data
}

#[test]
fn test_hl_block_reading() {
    // Create test file with HL -> DL -> DT structure
    let mut file = NamedTempFile::new().unwrap();
    
    // Write HL block at offset 0
    let hl_data = create_test_hl_block();
    file.write_all(&hl_data).unwrap();
    
    // Write DL block at offset 64
    file.seek(std::io::SeekFrom::Start(64)).unwrap();
    let dl_data = create_test_dl_block();
    file.write_all(&dl_data).unwrap();
    
    // Write DT block at offset 88
    file.seek(std::io::SeekFrom::Start(88)).unwrap();
    let dt_data = create_test_dt_block();
    file.write_all(&dt_data).unwrap();
    
    file.flush().unwrap();
    
    // Read the file
    let file_data = std::fs::read(file.path()).unwrap();
    
    // Test HL block reading
    let (pos, hl_block) = Hlblock::read(&file_data, 0, true);
    assert_eq!(pos, 50);
    assert_eq!(hl_block.hl_dl_first, 64);
    assert_eq!(hl_block.hl_zip_type, ZipType::None);
    
    // Test data block type reading
    let data_block = DataBlockType::read(&file_data, 0, true);
    match data_block {
        DataBlockType::HList(block) => {
            assert_eq!(block.hl_dl_first, 64);
        }
        _ => panic!("Expected HList block type"),
    }
}

#[test]
fn test_hl_block_with_dz() {
    // Create test file with HL -> DL -> DZ structure
    let mut file = NamedTempFile::new().unwrap();
    
    // Write HL block at offset 0
    let hl_data = create_test_hl_block();
    file.write_all(&hl_data).unwrap();
    
    // Write DL block at offset 64
    file.seek(std::io::SeekFrom::Start(64)).unwrap();
    let dl_data = create_test_dl_block();
    file.write_all(&dl_data).unwrap();
    
    // Write DZ block at offset 88
    file.seek(std::io::SeekFrom::Start(88)).unwrap();
    let dz_data = create_test_dz_block();
    file.write_all(&dz_data).unwrap();
    
    file.flush().unwrap();
    
    // Read the file
    let file_data = std::fs::read(file.path()).unwrap();
    
    // Test HL block reading
    let (pos, hl_block) = Hlblock::read(&file_data, 0, true);
    assert_eq!(pos, 50);
    assert_eq!(hl_block.hl_dl_first, 64);
    assert_eq!(hl_block.hl_zip_type, ZipType::None);
    
    // Test data block type reading
    let data_block = DataBlockType::read(&file_data, 0, true);
    match data_block {
        DataBlockType::HList(block) => {
            assert_eq!(block.hl_dl_first, 64);
        }
        _ => panic!("Expected HList block type"),
    }
}

#[test]
fn test_nested_hl_blocks() {
    // Create test file with HL -> HL -> DL -> DT structure
    let mut file = NamedTempFile::new().unwrap();
    
    // Write first HL block at offset 0
    let hl_data = create_test_hl_block();
    file.write_all(&hl_data).unwrap();
    
    // Write second HL block at offset 64
    file.seek(std::io::SeekFrom::Start(64)).unwrap();
    let hl_data = create_test_hl_block();
    file.write_all(&hl_data).unwrap();
    
    // Write DL block at offset 114
    file.seek(std::io::SeekFrom::Start(114)).unwrap();
    let dl_data = create_test_dl_block();
    file.write_all(&dl_data).unwrap();
    
    // Write DT block at offset 138
    file.seek(std::io::SeekFrom::Start(138)).unwrap();
    let dt_data = create_test_dt_block();
    file.write_all(&dt_data).unwrap();
    
    file.flush().unwrap();
    
    // Read the file
    let file_data = std::fs::read(file.path()).unwrap();
    
    // Test first HL block reading
    let (pos, hl_block) = Hlblock::read(&file_data, 0, true);
    assert_eq!(pos, 50);
    assert_eq!(hl_block.hl_dl_first, 64);
    
    // Test second HL block reading
    let (pos, hl_block) = Hlblock::read(&file_data, 64, true);
    assert_eq!(pos, 114);
    assert_eq!(hl_block.hl_dl_first, 114);
}

#[test]
fn test_hl_block_data_reading() {
    // Create test file with HL -> DL -> DL -> DT structure
    let mut file = NamedTempFile::new().unwrap();
    
    // Write HL block at offset 0
    let hl_data = create_test_hl_block();
    file.write_all(&hl_data).unwrap();
    
    // Write first DL block at offset 64
    file.seek(std::io::SeekFrom::Start(64)).unwrap();
    let dl_data = create_test_dl_block();
    file.write_all(&dl_data).unwrap();
    
    // Write second DL block at offset 88
    file.seek(std::io::SeekFrom::Start(88)).unwrap();
    let dl_data = create_test_dl_block();
    file.write_all(&dl_data).unwrap();
    
    // Write DT blocks with test data
    // First DT block at offset 112
    file.seek(std::io::SeekFrom::Start(112)).unwrap();
    let mut dt_data = create_test_dt_block();
    dt_data[24..28].copy_from_slice(&[1, 2, 3, 4]); // Test data for first DT block
    file.write_all(&dt_data).unwrap();
    
    // Second DT block at offset 136
    file.seek(std::io::SeekFrom::Start(136)).unwrap();
    let mut dt_data = create_test_dt_block();
    dt_data[24..28].copy_from_slice(&[5, 6, 7, 8]); // Test data for second DT block
    file.write_all(&dt_data).unwrap();
    
    file.flush().unwrap();
    
    // Read the file
    let file_data = std::fs::read(file.path()).unwrap();
    
    // Test HL block reading and data extraction
    let (pos, hl_block) = Hlblock::read(&file_data, 0, true);
    assert_eq!(pos, 50);
    assert_eq!(hl_block.hl_dl_first, 64);
    
    // Read the data from the HL block
    let data = hl_block.read_data(&file_data, true);
    
    // Verify the data
    assert_eq!(data.len(), 8); // Should have 4 bytes from each DT block
    assert_eq!(&data[0..4], &[1, 2, 3, 4]); // First DT block data
    assert_eq!(&data[4..8], &[5, 6, 7, 8]); // Second DT block data
} 