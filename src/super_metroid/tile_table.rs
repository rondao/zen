use std::convert::TryInto;

use crate::graphics::gfx::TILE_SIZE;

pub const TILE_TABLE_SIZE: usize = 32;
pub const TILES_BY_BLOCK: usize = 4;
pub const BLOCK_SIZE: usize = TILE_SIZE * 2;

pub type TileTable = Vec<Tile>;

#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub struct Tile {
    pub y_flip: bool,
    pub x_flip: bool,
    pub unknown: bool,
    pub sub_palette: u8,
    pub gfx_index: u16,
}

#[rustfmt::skip]
pub fn from_bytes(source: &[u8]) -> TileTable {
    source
        .chunks(2)
        .map(|tile_data| {
            let two_bytes =  u16::from_le_bytes(tile_data.try_into().unwrap_or( [tile_data[0], 0]));
            Tile {
                y_flip:       (two_bytes & 0b1000_0000_0000_0000) != 0,
                x_flip:       (two_bytes & 0b0100_0000_0000_0000) != 0,
                unknown:      (two_bytes & 0b0010_0000_0000_0000) != 0,
                sub_palette: ((two_bytes & 0b0001_1100_0000_0000) >> 10) as u8,
                gfx_index:    (two_bytes & 0b0000_0011_1111_1111) as u16,
            }
        })
        .collect()
}

mod tests {
    use super::*;

    /// Load a tile table from bytes, each tile is two bytes.
    #[test]
    fn load_tile_table_from_bytes() {
        let data: Vec<u8> = [
            [0b0000_0000, 0b0_0_0_000_00],
            [0b1010_0101, 0b0_1_0_101_01],
            [0b0101_1010, 0b1_0_1_010_10],
            [0b1111_1111, 0b1_1_1_111_11],
        ]
        .concat();

        let expected_tile_table = [
            Tile {
                y_flip: false,
                x_flip: false,
                unknown: false,
                sub_palette: 0,
                gfx_index: 0,
            },
            Tile {
                y_flip: false,
                x_flip: true,
                unknown: false,
                sub_palette: 0b101,
                gfx_index: 0b01_1010_0101,
            },
            Tile {
                y_flip: true,
                x_flip: false,
                unknown: true,
                sub_palette: 0b010,
                gfx_index: 0b10_0101_1010,
            },
            Tile {
                y_flip: true,
                x_flip: true,
                unknown: true,
                sub_palette: 0b111,
                gfx_index: 0b11_1111_1111,
            },
        ];

        assert_eq!(from_bytes(&data), expected_tile_table);
    }
}
