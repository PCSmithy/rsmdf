use super::{
    block::{Block, DataBlock, LinkedBlock},
    block_header::BlockHeader,
    dl_block::Dlblock,
    dt_block::Dtblock,
    dz_block::Dzblock,
    hl_block::Hlblock,
};

pub enum DataBlockType {
    Block(Dtblock),
    BlockComp(Dzblock),
    List(Dlblock),
    HList(Hlblock),
}

impl DataBlockType {
    pub fn data_array(&self, stream: &[u8], little_endian: bool) -> Vec<u8> {
        match self {
            Self::Block(block) => {
                println!("Reading from DT block");
                block.data_array(stream, little_endian)
            },
            Self::BlockComp(block) => {
                println!("Reading from DZ block (compressed)");
                block.data_array(stream, little_endian)
            },
            Self::List(block) => {
                println!("Reading from DL block (list)");
                let dl_list = block.list(stream, little_endian);
                println!("Found {} blocks in list", dl_list.len());

                let mut data = Vec::new();

                for (i, dl) in dl_list.iter().enumerate() {
                    let mut block_data = dl.data_array(stream, little_endian);
                    println!("Block {} contains {} bytes", i, block_data.len());
                    data.append(&mut block_data);
                }

                data
            },
            Self::HList(block) => {
                println!("Reading from HL block (hierarchical list)");
                // TODO: Implement data_array for Hlblock
                Vec::new()
            }
        }
    }

    pub fn read(stream: &[u8], position: usize, little_endian: bool) -> Self {
        let (_pos, header) = BlockHeader::read(stream, position, little_endian);

        let data_block = match std::str::from_utf8(&header.id).unwrap() {
            "##DT" => {
                let (_pos, block) = Dtblock::read(stream, position, little_endian);
                Self::Block(block)
            }
            "##DZ" => {
                let (_pos, block) = Dzblock::read(stream, position, little_endian);
                Self::BlockComp(block)
            }
            "##DL" => {
                let (_pos, block) = Dlblock::read(stream, position, little_endian);
                Self::List(block)
            }
            "##HL" => {
                let (_pos, block) = Hlblock::read(stream, position, little_endian);
                Self::HList(block)
            }
            _ => panic!("Error: wrong block type for data block"),
        };

        data_block
    }
}
