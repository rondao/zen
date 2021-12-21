use std::convert::TryInto;

#[derive(Debug, Default, Clone, Copy)]
pub struct Block {
    pub block_type: u8,
    pub y_flip: bool,
    pub x_flip: bool,
    pub block_number: u16,
}

type BtsBlock = u8;

#[derive(Debug, Default, Clone)]
pub struct LevelData {
    pub layer1: Vec<Block>,
    pub bts: Vec<BtsBlock>,
    pub layer2: Option<Vec<Block>>,
}

/// Room format reference: https://wiki.metroidconstruction.com/doku.php?id=super:technical_information:data_structures#level_data
pub fn from_bytes(source: &[u8], has_layer2: bool) -> LevelData {
    let layer_size = u16::from_le_bytes([source[0], source[1]]) as usize;
    let number_of_blocks = layer_size / 2;

    let source = &source[2..];
    let layer1: Vec<Block> = layer_from_bytes(&source[..number_of_blocks * 2]); // Each block is 2 bytes.

    let source = &source[number_of_blocks * 2..];
    let bts: Vec<BtsBlock> = source[..number_of_blocks].iter().copied().collect();

    let source = &source[number_of_blocks..];
    let layer2 = if has_layer2 {
        Some(layer_from_bytes(&source[..number_of_blocks * 2])) // Each block is 2 bytes.
    } else {
        None
    };

    LevelData {
        layer1,
        bts,
        layer2,
    }
}

#[rustfmt::skip]
fn layer_from_bytes(source: &[u8]) -> Vec<Block> {
    source
        .chunks(2)
        .map(|block_data| {
            let two_bytes = u16::from_le_bytes(block_data.try_into().unwrap());
            Block {
                block_type:  ((two_bytes & 0b1111_0000_0000_0000) >> 12) as u8,
                y_flip:       (two_bytes & 0b0000_1000_0000_0000) != 0,
                x_flip:       (two_bytes & 0b0000_0100_0000_0000) != 0,
                block_number: (two_bytes & 0b0000_0011_1111_1111) as u16,
            }
        })
        .collect()
}
