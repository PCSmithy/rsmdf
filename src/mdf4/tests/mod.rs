use crate::mdf4::{block::Block, block_header::BlockHeader, md_block::Mdblock};

#[test]
fn test_block_header() {
    let bytes = [
        b'#', b'#', b'M', b'D', 0x32, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    ];

    let (pos, header) = BlockHeader::read(&bytes, 0, true).unwrap();
    assert_eq!(pos, 24);
    assert_eq!(&header.id, b"##MD");
    assert_eq!(header.length, 50);
    assert_eq!(header.link_count, 0);
}

#[test]
fn test_md_block() {
    let bytes = [
        b'#', b'#', b'M', b'D', 0x32, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, b'T', b'h', b'i', b's', b' ', b'i', b's', b' ',
        b's', b'o', b'm', b'e', b' ', b'm', b'e', b'a', b's', b'u', b'r', b'e', b'm', b'e', b'n', b't',
        0x00, 0x00,
    ];

    let (pos, md_block) = Mdblock::read(&bytes, 0, true).unwrap();
    assert_eq!(pos, 50);
    assert_eq!(md_block.data(), "This is some measurement");
}

#[test]
fn test_invalid_block_header() {
    let bytes = [0u8; 24];
    assert!(BlockHeader::read(&bytes, 0, true).is_err());
}

#[test]
fn test_invalid_md_block() {
    let bytes = [0u8; 50];
    assert!(Mdblock::read(&bytes, 0, true).is_err());
} 