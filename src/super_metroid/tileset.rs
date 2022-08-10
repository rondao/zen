use crate::graphics::{
    gfx::{Gfx, TILE_SIZE},
    palette::{Palette, Rgb888},
};

use super::tile_table::{TileTable, TILES_BY_BLOCK, TILE_TABLE_SIZE};

pub const TILESET_BLOCK_SIZE: usize = 16;
pub const TILESET_DATA_SIZE: usize = 9;
pub const NUMBER_OF_TILESETS: usize = 0x1D;

#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub struct Tileset {
    pub palette: u32,
    pub graphic: u32,
    pub tile_table: u32,
    pub use_cre: bool,
}

impl Tileset {
    pub fn to_bytes(&self) -> Vec<u8> {
        // Values are 'u32', but should be 'u24'. So we remove the extra byte.
        [
            &self.tile_table.to_le_bytes()[..3],
            &self.graphic.to_le_bytes()[..3],
            &self.palette.to_le_bytes()[..3],
        ]
        .concat()
    }
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
    let size = tileset_size();
    let mut colors = Vec::with_capacity(size[0] * size[1]);

    // Add colors of each row of blocks a time. Each block is composed of 'TILES_BY_BLOCK' smaller 'TileGfx'.
    for row_of_blocks in tile_table.chunks(TILE_TABLE_SIZE * TILES_BY_BLOCK) {
        // Each block has two rows of tiles. Let's add colors for the top row of tiles, and then the bottow row.
        for tile_row in 0..=1 {
            // Each tile have 'TILE_SIZE' rows of colors. Let's add all colors of each row at a time.
            for tile_color_row in (0..TILE_SIZE * TILE_SIZE).step_by(TILE_SIZE) {
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

pub fn tileset_size() -> [usize; 2] {
    [
        (TILESET_BLOCK_SIZE * TILE_TABLE_SIZE),
        (TILESET_BLOCK_SIZE * TILE_TABLE_SIZE),
    ]
}

#[cfg(test)]
mod tests {
    use std::convert::TryInto;

    use crate::{
        graphics::{
            gfx::TileGfx,
            palette::{Bgr555, SubPalette, COLORS_BY_SUB_PALETTE, NUMBER_OF_SUB_PALETTES},
        },
        super_metroid::tile_table::Tile,
    };

    use super::*;

    /// Load a tileset from bytes, each tileset is [TILESET_DATA_SIZE].
    /// Convert a tileset into bytes.
    #[test]
    fn load_tileset_from_bytes() {
        let tilesets_data: Vec<u8> = [
            // Tileset 1
            [0b01010101, 0b00000000, 0b00000000], // Tiletable 1
            [0b10101010, 0b00100010, 0b00000000], // Graphic 1
            [0b10111101, 0b10011001, 0b01100110], // Palette 1
            // Tileset 2
            [0b00000000, 0b00000000, 0b01010101], // Tiletable 2
            [0b00000000, 0b00100010, 0b10101010], // Graphic 2
            [0b01100110, 0b10011001, 0b10111101], // Palette 2
        ]
        .concat();

        let tilesets = vec![
            Tileset {
                palette: 0b01100110_10011001_10111101,
                graphic: 0b00100010_10101010,
                tile_table: 0b01010101,
                use_cre: true,
            },
            Tileset {
                palette: 0b10111101_10011001_01100110,
                graphic: 0b10101010_00100010_00000000,
                tile_table: 0b01010101_00000000_00000000,
                use_cre: true,
            },
        ];
        assert_eq!(from_bytes(&tilesets_data), tilesets);

        let tilesets_to_bytes = tilesets.iter().fold(Vec::new(), |mut accum, tileset| {
            accum.extend(tileset.to_bytes());
            accum
        });
        assert_eq!(tilesets_to_bytes, tilesets_data);
    }

    /// Convert tileset to colors.
    #[test]
    fn convert_tileset_to_colors() {
        // == Palette ==
        let sub_palette_0: [Bgr555; COLORS_BY_SUB_PALETTE] = [
            Bgr555 {
                r: 0b000_00001, // 1
                g: 0b000_00010, // 2
                b: 0b000_00011, // 3
                u: 0,
            },
            Bgr555 {
                r: 0b000_00100, // 4
                g: 0b000_00101, // 5
                b: 0b000_00110, // 6
                u: 0,
            },
        ]
        .repeat(COLORS_BY_SUB_PALETTE / 2)
        .try_into()
        .unwrap();

        let sub_palette_1: [Bgr555; COLORS_BY_SUB_PALETTE] = [
            Bgr555 {
                r: 0b000_00111, // 7
                g: 0b000_01000, // 8
                b: 0b000_01001, // 9
                u: 0,
            },
            Bgr555 {
                r: 0b000_01010, // 10
                g: 0b000_01011, // 11
                b: 0b000_01100, // 12
                u: 0,
            },
        ]
        .repeat(COLORS_BY_SUB_PALETTE / 2)
        .try_into()
        .unwrap();

        let sub_palettes: [SubPalette; NUMBER_OF_SUB_PALETTES] = [
            SubPalette {
                colors: sub_palette_0,
            },
            SubPalette {
                colors: sub_palette_1,
            },
        ]
        .repeat(NUMBER_OF_SUB_PALETTES / 2)
        .try_into()
        .unwrap();

        let palette = Palette { sub_palettes };

        // == Gfx ==
        #[rustfmt::skip]
        let graphics = Gfx { tiles: vec![
            TileGfx { colors: [
                1, 1, 1, 1, 0, 0, 0, 0,
                1, 1, 1, 1, 0, 0, 0, 0,
                0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 0, 0, 0, 1, 1,
                0, 0, 0, 0, 0, 0, 1, 1,
                0, 0, 0, 0, 0, 0, 1, 1,
                0, 0, 0, 0, 0, 0, 1, 1,
            ]},
            TileGfx { colors: [
                1, 1, 1, 0, 0, 0, 0, 0,
                1, 1, 1, 0, 0, 0, 0, 0,
                0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 0, 0, 0, 1, 1,
                0, 0, 0, 0, 0, 0, 1, 1,
                0, 0, 0, 0, 0, 0, 1, 1,
            ]},
        ]};

        // == Tile Table ==
        let tile_table = vec![
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
                sub_palette: 0,
                gfx_index: 1,
            },
            Tile {
                y_flip: true,
                x_flip: false,
                unknown: true,
                sub_palette: 1,
                gfx_index: 1,
            },
            Tile {
                y_flip: true,
                x_flip: true,
                unknown: true,
                sub_palette: 1,
                gfx_index: 0,
            },
        ];

        let p0c0 = Rgb888 { r: 8, g: 16, b: 24 };
        #[rustfmt::skip]
        let p0c1 = Rgb888 { r: 32, g: 40, b: 48 };
        #[rustfmt::skip]
        let p1c0 = Rgb888 { r: 56, g: 64, b: 72 };
        #[rustfmt::skip]
        let p1c1 = Rgb888 { r: 80, g: 88, b: 96 };

        #[rustfmt::skip]
        let expected_colors = [
            // TileGfx 0, P0, No Flip                       | TileGfx 1, P0, H Flip
            p0c1, p0c1, p0c1, p0c1, p0c0, p0c0, p0c0, p0c0,   p0c0, p0c0, p0c0, p0c0, p0c0, p0c1, p0c1, p0c1,
            p0c1, p0c1, p0c1, p0c1, p0c0, p0c0, p0c0, p0c0,   p0c0, p0c0, p0c0, p0c0, p0c0, p0c1, p0c1, p0c1,
            p0c0, p0c0, p0c0, p0c0, p0c0, p0c0, p0c0, p0c0,   p0c0, p0c0, p0c0, p0c0, p0c0, p0c0, p0c0, p0c0,
            p0c0, p0c0, p0c0, p0c0, p0c0, p0c0, p0c0, p0c0,   p0c0, p0c0, p0c0, p0c0, p0c0, p0c0, p0c0, p0c0,
            p0c0, p0c0, p0c0, p0c0, p0c0, p0c0, p0c1, p0c1,   p0c0, p0c0, p0c0, p0c0, p0c0, p0c0, p0c0, p0c0, 
            p0c0, p0c0, p0c0, p0c0, p0c0, p0c0, p0c1, p0c1,   p0c1, p0c1, p0c0, p0c0, p0c0, p0c0, p0c0, p0c0,
            p0c0, p0c0, p0c0, p0c0, p0c0, p0c0, p0c1, p0c1,   p0c1, p0c1, p0c0, p0c0, p0c0, p0c0, p0c0, p0c0,
            p0c0, p0c0, p0c0, p0c0, p0c0, p0c0, p0c1, p0c1,   p0c1, p0c1, p0c0, p0c0, p0c0, p0c0, p0c0, p0c0,
            // TileGfx 1, P1, V Flip                        | TileGfx 0, P1, VH Flip
            p1c0, p1c0, p1c0, p1c0, p1c0, p1c0, p1c1, p1c1,   p1c1, p1c1, p1c0, p1c0, p1c0, p1c0, p1c0, p1c0,
            p1c0, p1c0, p1c0, p1c0, p1c0, p1c0, p1c1, p1c1,   p1c1, p1c1, p1c0, p1c0, p1c0, p1c0, p1c0, p1c0,
            p1c0, p1c0, p1c0, p1c0, p1c0, p1c0, p1c1, p1c1,   p1c1, p1c1, p1c0, p1c0, p1c0, p1c0, p1c0, p1c0,
            p1c0, p1c0, p1c0, p1c0, p1c0, p1c0, p1c0, p1c0,   p1c1, p1c1, p1c0, p1c0, p1c0, p1c0, p1c0, p1c0,
            p1c0, p1c0, p1c0, p1c0, p1c0, p1c0, p1c0, p1c0,   p1c0, p1c0, p1c0, p1c0, p1c0, p1c0, p1c0, p1c0,
            p1c0, p1c0, p1c0, p1c0, p1c0, p1c0, p1c0, p1c0,   p1c0, p1c0, p1c0, p1c0, p1c0, p1c0, p1c0, p1c0,
            p1c1, p1c1, p1c1, p1c0, p1c0, p1c0, p1c0, p1c0,   p1c0, p1c0, p1c0, p1c0, p1c1, p1c1, p1c1, p1c1,
            p1c1, p1c1, p1c1, p1c0, p1c0, p1c0, p1c0, p1c0,   p1c0, p1c0, p1c0, p1c0, p1c1, p1c1, p1c1, p1c1,
        ];

        assert_eq!(
            tileset_to_colors(&tile_table, &palette, &graphics),
            expected_colors
        );
    }
}
