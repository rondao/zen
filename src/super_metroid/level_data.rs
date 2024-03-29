use std::convert::TryInto;

use crate::{
    graphics::{
        gfx::{Gfx, TILE_SIZE},
        palette::{Palette, Rgb888},
        IndexedColor,
    },
    ParseError,
};

use super::tile_table::{TileTable, BLOCK_SIZE, TILES_BY_BLOCK};

#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub struct Block {
    pub block_type: BlockType, // Specifies the primary type of the block.
    pub y_flip: bool,          // Flips the graphics of the block at Y axis.
    pub x_flip: bool,          // Flips the graphics of the block at X axis.
    pub block_number: u16,     // Specifies the index of the block into the tile table.
}

#[derive(Debug, Default, Clone, Copy, Hash, Eq, PartialEq)]
pub enum BlockType {
    #[default]
    Air,
    Slope,
    AirSpike,
    AirSpecial,
    AirShootable,
    HorizontalExtension,
    AirUnused,
    AirBombable,
    Solid,
    Door,
    SolidSpike,
    SolidSpecial,
    SolidShootable,
    VerticalExtension,
    SolidGrapple,
    SolidBombable,
}

pub type BtsBlock = u8;

pub const BLOCKS_PER_SCREEN: usize = 16;

#[derive(Debug, Default, Clone, PartialEq)]
pub struct LevelData {
    pub layer1: Vec<Block>,
    pub bts: Vec<BtsBlock>,
    pub layer2: Option<Vec<Block>>,
}

impl From<usize> for BlockType {
    fn from(value: usize) -> Self {
        match value {
            0x0 => Self::Air,
            0x1 => Self::Slope,
            0x2 => Self::AirSpike,
            0x3 => Self::AirSpecial,
            0x4 => Self::AirShootable,
            0x5 => Self::HorizontalExtension,
            0x6 => Self::AirUnused,
            0x7 => Self::AirBombable,
            0x8 => Self::Solid,
            0x9 => Self::Door,
            0xA => Self::SolidSpike,
            0xB => Self::SolidSpecial,
            0xC => Self::SolidShootable,
            0xD => Self::VerticalExtension,
            0xE => Self::SolidGrapple,
            0xF => Self::SolidBombable,
            _ => Self::Air,
        }
    }
}

impl From<u16> for BlockType {
    fn from(value: u16) -> Self {
        (value as usize).into()
    }
}

impl From<u8> for BlockType {
    fn from(value: u8) -> Self {
        (value as usize).into()
    }
}

impl From<i32> for BlockType {
    fn from(value: i32) -> Self {
        (value as usize).into()
    }
}

/// Level Data format reference: https://wiki.metroidconstruction.com/doku.php?id=super:technical_information:data_structures#level_data
pub fn load_from_bytes(source: &[u8], has_layer2: bool) -> Result<LevelData, ParseError> {
    if source.len() < 2 {
        return Err(ParseError);
    };

    let layer_size = u16::from_le_bytes([source[0], source[1]]) as usize;
    let number_of_blocks = layer_size / 2;

    let source = &source[2..];
    if source.len()
        < number_of_blocks * 2 + number_of_blocks + number_of_blocks * 2 * (has_layer2 as usize)
    {
        return Err(ParseError);
    };

    let layer1: Vec<Block> = layer_from_bytes(&source[..number_of_blocks * 2]); // Each block is 2 bytes.

    let source = &source[number_of_blocks * 2..];
    let bts: Vec<BtsBlock> = source[..number_of_blocks].iter().copied().collect();

    let source = &source[number_of_blocks..];
    let layer2 = if has_layer2 {
        Some(layer_from_bytes(&source[..number_of_blocks * 2])) // Each block is 2 bytes.
    } else {
        None
    };

    Ok(LevelData {
        layer1,
        bts,
        layer2,
    })
}

#[rustfmt::skip]
fn layer_from_bytes(source: &[u8]) -> Vec<Block> {
    source
        .chunks(2)
        .map(|block_data| {
            let two_bytes = u16::from_le_bytes(block_data.try_into().unwrap());
            Block {
                block_type:  ((two_bytes & 0b1111_0000_0000_0000) >> 12).into(),
                y_flip:       (two_bytes & 0b0000_1000_0000_0000) != 0,
                x_flip:       (two_bytes & 0b0000_0100_0000_0000) != 0,
                block_number: (two_bytes & 0b0000_0011_1111_1111) as u16,
            }
        })
        .collect()
}

impl LevelData {
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut output = Vec::new();

        let layer_size = (self.layer1.len() * 2) as u16;
        output.extend(layer_size.to_le_bytes());

        output.extend(self.layer1.iter().fold(Vec::new(), |mut acc, block| {
            acc.extend(block.to_bytes());
            acc
        }));
        output.extend(self.bts.iter());
        if let Some(layer2) = &self.layer2 {
            output.append(layer2.iter().fold(&mut Vec::new(), |acc, block| {
                acc.extend(block.to_bytes());
                acc
            }));
        }

        output
    }

    pub fn to_colors<'s, 'p: 's>(
        &'s self,
        size: (usize, usize),
        tile_table: &TileTable,
        palette: &'p Palette,
        graphics: &Gfx,
    ) -> impl Iterator<Item = Rgb888> + '_ {
        self.to_indexed_colors(size, tile_table, graphics)
            .into_iter()
            .map(move |indexed_color: IndexedColor| {
                palette.sub_palettes[indexed_color.sub_palette].colors[indexed_color.index].into()
            })
    }

    pub fn to_indexed_colors(
        &self,
        size: (usize, usize),
        tile_table: &TileTable,
        graphics: &Gfx,
    ) -> Vec<IndexedColor> {
        let pixels_per_side = BLOCKS_PER_SCREEN * BLOCK_SIZE;
        let mut indexed_colors =
            vec![IndexedColor::default(); pixels_per_side * size.0 * pixels_per_side * size.1];

        if let Some(layer2) = &self.layer2 {
            self.layer_to_indexed_colors(size, &mut indexed_colors, &layer2, tile_table, graphics);
        }
        self.layer_to_indexed_colors(
            size,
            &mut indexed_colors,
            &self.layer1,
            tile_table,
            graphics,
        );

        indexed_colors
    }

    fn layer_to_indexed_colors(
        &self,
        size: (usize, usize),
        indexed_colors: &mut Vec<IndexedColor>,
        blocks: &Vec<Block>,
        tile_table: &TileTable,
        graphics: &Gfx,
    ) {
        for i_block in 0..(BLOCKS_PER_SCREEN * size.0 * BLOCKS_PER_SCREEN * size.1) {
            let block = blocks[i_block];

            let tileset_tile = block.block_number as usize * TILES_BY_BLOCK;
            let mut tiles: Vec<_> = tile_table[tileset_tile..tileset_tile + TILES_BY_BLOCK]
                .iter()
                .copied()
                .collect();

            if block.x_flip {
                tiles.swap(0, 1);
                tiles.swap(2, 3);
            }
            if block.y_flip {
                tiles.swap(0, 2);
                tiles.swap(1, 3);
            }

            for (i_tile, tile) in tiles.iter().enumerate() {
                for (i_color, index_color) in graphics.tiles[tile.gfx_index as usize]
                    .flip((tile.x_flip ^ block.x_flip, tile.y_flip ^ block.y_flip))
                    .iter()
                    .enumerate()
                {
                    if *index_color != 0 {
                        let pixel_per_width = BLOCKS_PER_SCREEN * size.0;

                        #[rustfmt::skip]
                        let y = (i_block / pixel_per_width) * pixel_per_width * BLOCK_SIZE * BLOCK_SIZE
                              + (i_tile / 2)                * pixel_per_width * BLOCK_SIZE * TILE_SIZE
                              + (i_color / TILE_SIZE)       * pixel_per_width * BLOCK_SIZE;
                        #[rustfmt::skip]
                        let x = (i_block % pixel_per_width) * BLOCK_SIZE
                              + (i_tile % 2)                * TILE_SIZE
                              + (i_color % TILE_SIZE);

                        indexed_colors[y + x] = IndexedColor {
                            index: *index_color as usize,
                            sub_palette: tile.sub_palette as usize,
                        };
                    }
                }
            }
        }
    }
}

impl Block {
    pub fn to_bytes(&self) -> [u8; 2] {
        let y_flip = if self.y_flip { 1 } else { 0 };
        let x_flip = if self.x_flip { 1 } else { 0 };

        (self.block_number | ((self.block_type as u16) << 12) | (y_flip << 11) | (x_flip << 10))
            .to_le_bytes()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Load one level layer from bytes.
    #[test]
    fn load_one_level_layer_from_bytes() {
        let expected_blocks = vec![
            Block {
                block_type: 0b0000.into(),
                y_flip: false,
                x_flip: false,
                block_number: 0b00_0000_0000,
            },
            Block {
                block_type: 0b0101.into(),
                y_flip: true,
                x_flip: false,
                block_number: 0b01_0101_0101,
            },
            Block {
                block_type: 0b1010.into(),
                y_flip: false,
                x_flip: true,
                block_number: 0b10_1010_1010,
            },
            Block {
                block_type: 0b1111.into(),
                y_flip: true,
                x_flip: true,
                block_number: 0b11_1111_1111,
            },
        ];

        #[rustfmt::skip]
        let data = [
            0b00000000, 0b0000_0_0_00,
            0b01010101, 0b0101_1_0_01,
            0b10101010, 0b1010_0_1_10,
            0b11111111, 0b1111_1_1_11,
        ];

        assert_eq!(layer_from_bytes(&data), expected_blocks);
    }

    /// Load a full level data from bytes.
    /// Test level data with one and two layers.
    #[test]
    fn load_level_data_from_bytes() {
        let mut expected_level_data = LevelData {
            layer1: vec![
                Block {
                    block_type: 0b0000.into(),
                    y_flip: false,
                    x_flip: false,
                    block_number: 0b00_0000_0000,
                },
                Block {
                    block_type: 0b1111.into(),
                    y_flip: true,
                    x_flip: true,
                    block_number: 0b11_1111_1111,
                },
            ],
            bts: vec![0b1010_1010, 0b0101_0101],
            layer2: None,
        };

        #[rustfmt::skip]
        let data_one_layer = vec![
            0b0000_0100, 0b0000_0000,  // Layer size in bytes. So 4, for 2 blocks.
            0b00000000, 0b0000_0_0_00, // Layer_1 block 01
            0b11111111, 0b1111_1_1_11, // Layer_1 block 02
            0b1010_1010, // Bts block 01
            0b0101_0101, // Bts block 02
        ];

        let level_result = load_from_bytes(&data_one_layer, false);
        assert!(level_result.is_ok());
        assert_eq!(level_result.unwrap(), expected_level_data);

        // Add a layer 2 for another test.
        expected_level_data.layer2 = Some(vec![
            Block {
                block_type: 0b0101.into(),
                y_flip: true,
                x_flip: false,
                block_number: 0b01_0101_0101,
            },
            Block {
                block_type: 0b1010.into(),
                y_flip: false,
                x_flip: true,
                block_number: 0b10_1010_1010,
            },
        ]);

        #[rustfmt::skip]
        let data_two_layer = [
            data_one_layer,
            vec![0b01010101, 0b0101_1_0_01,
                 0b10101010, 0b1010_0_1_10,],
        ]
        .concat();

        let level_result = load_from_bytes(&data_two_layer, true);
        assert!(level_result.is_ok());
        assert_eq!(level_result.unwrap(), expected_level_data);
    }

    /// Load a level with incorrect data size
    #[test]
    fn load_level_data_with_incorrect_data_size() {
        let data_empty: &[u8] = &[];
        let data_without_minimal_size: &[u8] = &[0xFF];
        let data_with_incorrect_size_header: &[u8] = &[0x02, 0x00];

        assert!(load_from_bytes(data_empty, false).is_err());
        assert!(load_from_bytes(data_without_minimal_size, false).is_err());
        assert!(load_from_bytes(data_with_incorrect_size_header, false).is_err());
    }
}
