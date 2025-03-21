use crate::mdf::{MDFType, MDFVersion, MDF};
use std::fs::File;
use std::io::Write;
use tempfile::NamedTempFile;

fn create_test_mdf3_file() -> NamedTempFile {
    let mut file = NamedTempFile::new().unwrap();
    
    // ID Block
    let id_block = [
        b'M', b'D', b'F', b' ', b' ', b' ', b' ', b' ',  // File ID
        b'3', b'.', b'0', b'0', b' ', b' ', b' ', b' ',  // Format ID
        b'R', b'S', b'M', b'D', b'F', b' ', b' ', b' ',  // Program ID
        0, 0,  // Default byte order (little endian)
        0, 0,  // Default float format (IEEE 754)
        0x2C, 0x01,  // Version number (300 = 3.00)
        0, 0,  // Code page (not used)
        0, 0,  // Reserved
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,  // Reserved
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0  // Reserved
    ];

    // HD Block
    let hd_block = [
        b'H', b'D',  // Block type
        0x00, 0xD8,  // Block size
        0, 0, 0, 0,  // Pointer to first DG block (none)
        0, 0, 0, 0,  // Pointer to file comment (none)
        0, 0, 0, 0,  // Pointer to program block (none)
        0, 0,  // Number of data groups
    ];
    let hd_padding = vec![0u8; 0xD8 - hd_block.len()];  // Pad to block size

    file.write_all(&id_block).unwrap();
    file.write_all(&hd_block).unwrap();
    file.write_all(&hd_padding).unwrap();
    file.flush().unwrap();
    file
}

#[test]
fn test_mdf3_version_detection() {
    let test_file = create_test_mdf3_file();
    let version = MDFType::check_version(test_file.path().to_str().unwrap());
    assert_eq!(version, MDFVersion::MDF3);
}

#[test]
fn test_mdf3_file_loading() {
    let test_file = create_test_mdf3_file();
    let mdf = MDF::new(test_file.path().to_str().unwrap());
    assert!(!mdf.filepath.is_empty());
}

#[test]
fn test_mdf3_empty_channels() {
    let test_file = create_test_mdf3_file();
    let mut mdf = MDF::new(test_file.path().to_str().unwrap());
    mdf.read_all();
    let channels = mdf.channels();
    assert!(channels.is_empty(), "Empty MDF3 file should have no channels");
} 