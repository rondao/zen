use std::convert::TryInto;

use crate::graphics::gfx::TILE_SIZE;

pub const TILE_TABLE_SIZE: usize = 32;
pub const TILES_BY_BLOCK: usize = 4;
pub const BLOCK_SIZE: usize = TILE_SIZE * 2;

pub type TileTable = Vec<Tile>;

#[derive(Debug, Default, Clone, Copy)]
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
