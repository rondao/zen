use crate::graphics::{
    gfx::{Gfx, TILE_SIZE},
    palette::{Palette, Rgb888},
};

use super::tile_table::{TileTable, TILES_BY_BLOCK, TILE_TABLE_SIZE};

pub const TILESET_BLOCK_SIZE: usize = 16;
pub const TILESET_DATA_SIZE: usize = 9;
pub const NUMBER_OF_TILESETS: usize = 0x1D;

#[derive(Debug, Default, Clone, Copy)]
pub struct Tileset {
    pub palette: u32,
    pub graphic: u32,
    pub tile_table: u32,
    pub use_cre: bool,
}

#[rustfmt::skip]
pub fn from_bytes(source: &[u8]) -> Vec<Tileset> {
    source
        .chunks(TILESET_DATA_SIZE).enumerate()
        .map(|(index, data)| {
            Tileset {
                tile_table: u32::from_le_bytes([data[0], data[1], data[2], 0]),
                graphic:    u32::from_le_bytes([data[3], data[4], data[5], 0]),
                palette:    u32::from_le_bytes([data[6], data[7], data[8], 0]),
                use_cre:    !(0xF..=0x14).contains(&index),
            }
        })
        .collect()
}

pub fn tileset_to_colors(tile_table: &TileTable, palette: &Palette, graphics: &Gfx) -> Vec<Rgb888> {
    let mut colors = Vec::new();
    // Add colors of each row of blocks a time. Each block is composed of 'TILES_BY_BLOCK' smaller 'TileGfx'.
    for row_of_blocks in tile_table.chunks(TILE_TABLE_SIZE * TILES_BY_BLOCK) {
        // Each block has two rows of tiles. Let's add colors for the top row of tiles, and then the bottow row.
        for tile_row in 0..=1 {
            // Each tile have 'TILE_SIZE' rows of colors. Let's add all colors of each row at a time.
            for tile_color_row in (0..TILE_SIZE).map(|value| value * TILE_SIZE) {
                // Let's loop the top and bottom row of tiles for each row of blocks.
                for row_of_tiles in row_of_blocks.chunks(2).skip(tile_row).step_by(2) {
                    // For each tile, let's add one row of colors a time.
                    for tile in row_of_tiles {
                        colors.extend::<Vec<Rgb888>>(
                            graphics.tiles[tile.gfx_index as usize]
                                .flip((tile.x_flip, tile.y_flip))
                                [tile_color_row..tile_color_row + TILE_SIZE]
                                .iter()
                                .map(|index_color| {
                                    palette.sub_palettes[tile.sub_palette as usize].colors
                                        [*index_color as usize]
                                        .into()
                                })
                                .collect(),
                        )
                    }
                }
            }
        }
    }
    colors
}
