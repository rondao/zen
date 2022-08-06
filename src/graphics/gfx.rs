use std::convert::TryInto;

pub const TILE_SIZE: usize = 8;
pub const GFX_TILE_WIDTH: usize = 16;

#[derive(Debug, Default, Clone, PartialEq)]
pub struct Gfx {
    pub tiles: Vec<TileGfx>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TileGfx {
    pub colors: [u8; TILE_SIZE * TILE_SIZE],
}

impl TileGfx {
    /// Reference: https://sneslab.net/wiki/Graphics_Format
    ///  [r0, bp1], [r0, bp2], [r1, bp1], [r1, bp2], [r2, bp1], [r2, bp2], [r3, bp1], [r3, bp2]
    ///  [r4, bp1], [r4, bp2], [r5, bp1], [r5, bp2], [r6, bp1], [r6, bp2], [r7, bp1], [r7, bp2]
    ///  [r0, bp3], [r0, bp4], [r1, bp3], [r1, bp4], [r2, bp3], [r2, bp4], [r3, bp3], [r3, bp4]
    ///  [r4, bp3], [r4, bp4], [r5, bp3], [r5, bp4], [r6, bp3], [r6, bp4], [r7, bp3], [r7, bp4]
    fn tile_4bpp(source: &[u8]) -> TileGfx {
        let mut colors: Vec<u8> = Vec::with_capacity(TILE_SIZE * TILE_SIZE);

        // One half of the data has bits 0,1 and the other has bits 2,3 to compose one color.
        let (bitplane_01, bitplane_23) = source.split_at(source.len() / 2);
        // First half's first and second byte has 0 and 1 bits respectively.
        // Second half's first and second byte has 2 and 3 bits respectively.
        for (pixel_01, pixel_23) in bitplane_01.chunks(2).zip(bitplane_23.chunks(2)) {
            // Each 'i' bit of the 4 bytes collected at each iteration composes one color.
            //  [bp1, bp2, bp3, bp4]
            for i in (0..TILE_SIZE).rev() {
                colors.push(
                    (((pixel_01[0] >> i) & 1) << 0)
                        + (((pixel_01[1] >> i) & 1) << 1)
                        + (((pixel_23[0] >> i) & 1) << 2)
                        + (((pixel_23[1] >> i) & 1) << 3),
                )
            }
        }
        TileGfx {
            colors: colors.try_into().unwrap(),
        }
    }

    pub fn flip(&self, flip: (bool, bool)) -> [u8; TILE_SIZE * TILE_SIZE] {
        let mut colors = [0; TILE_SIZE * TILE_SIZE];
        for x in 0..TILE_SIZE {
            for y in 0..TILE_SIZE {
                let xt = if flip.0 { TILE_SIZE - 1 - x } else { x };
                let yt = if flip.1 { TILE_SIZE - 1 - y } else { y };
                colors[x + y * TILE_SIZE] = self.colors[xt + yt * TILE_SIZE]
            }
        }
        colors
    }
}

/// Palette format reference: https://georgjz.github.io/snesaa03/
pub fn from_4bpp(source: &[u8]) -> Gfx {
    Gfx {
        // Each Tile8 has 8 rows, and each row needs 4 bytes for the 8 row's colors.
        tiles: source
            .chunks(TILE_SIZE * 4)
            .map(TileGfx::tile_4bpp)
            .collect(),
    }
}

impl Gfx {
    pub fn to_indexed_colors(&self) -> Vec<u8> {
        let mut gfx_index_colors = Vec::new();
        // Loop each Tile row
        for row_of_tiles in self.tiles.chunks(GFX_TILE_WIDTH) {
            for tile_row_number in 0..TILE_SIZE {
                for tile in row_of_tiles.iter() {
                    let color_row_position = tile_row_number * TILE_SIZE;
                    gfx_index_colors
                        .extend(&tile.colors[color_row_position..color_row_position + TILE_SIZE]);
                }
            }
        }
        gfx_index_colors
    }

    pub fn size(&self) -> [usize; 2] {
        [
            (GFX_TILE_WIDTH * TILE_SIZE),
            (self.tiles.len() * TILE_SIZE / GFX_TILE_WIDTH),
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Load a single 8x8 tile from bytes with 4 bits per plane.
    #[test]
    fn load_tile_gfx_from_4bpp() {
        #[rustfmt::skip]
        let expected_tile_gfx = TileGfx { colors: [
            0b0001, 0b0010, 0b0011, 0b0100, 0b0101, 0b0110, 0b0111, 0b1000,
            0b0010, 0b0011, 0b0100, 0b0101, 0b0110, 0b0111, 0b1000, 0b1001,
            0b0011, 0b0100, 0b0101, 0b0110, 0b0111, 0b1000, 0b1001, 0b1010,
            0b0100, 0b0101, 0b0110, 0b0111, 0b1000, 0b1001, 0b1010, 0b1011,
            0b0101, 0b0110, 0b0111, 0b1000, 0b1001, 0b1010, 0b1011, 0b1100,
            0b0110, 0b0111, 0b1000, 0b1001, 0b1010, 0b1011, 0b1100, 0b1101,
            0b0111, 0b1000, 0b1001, 0b1010, 0b1011, 0b1100, 0b1101, 0b1110,
            0b1000, 0b1001, 0b1010, 0b1011, 0b1100, 0b1101, 0b1110, 0b1111,
        ]};

        #[rustfmt::skip]
        let tile_gfx_in_4bpp = [
            0b1010_1010, 0b0110_0110, // Row 0 (bits 1-2)
            0b0101_0101, 0b1100_1100, // Row 1 (bits 1-2)
            0b1010_1010, 0b1001_1001, // Row 2 (bits 1-2)
            0b0101_0101, 0b0011_0011, // Row 3 (bits 1-2)
            0b1010_1010, 0b0110_0110, // Row 4 (bits 1-2)
            0b0101_0101, 0b1100_1100, // Row 5 (bits 1-2)
            0b1010_1010, 0b1001_1001, // Row 6 (bits 1-2)
            0b0101_0101, 0b0011_0011, // Row 7 (bits 1-2)
            0b0001_1110, 0b0000_0001, // Row 0 (bits 3-4)
            0b0011_1100, 0b0000_0011, // Row 1 (bits 3-4)
            0b0111_1000, 0b0000_0111, // Row 2 (bits 3-4)
            0b1111_0000, 0b0000_1111, // Row 3 (bits 3-4)
            0b1110_0001, 0b0001_1111, // Row 4 (bits 3-4)
            0b1100_0011, 0b0011_1111, // Row 5 (bits 3-4)
            0b1000_0111, 0b0111_1111, // Row 6 (bits 3-4)
            0b0000_1111, 0b1111_1111, // Row 7 (bits 3-4)
        ];

        assert_eq!(TileGfx::tile_4bpp(&tile_gfx_in_4bpp), expected_tile_gfx);
    }

    /// Load a single 8x8 tile from bytes with 4 bits per plane.
    #[test]
    fn flip_tile_gfx_horizontally_vertically_and_both() {
        #[rustfmt::skip]
        let tile_gfx = TileGfx { colors: [
            0b0001, 0b0010, 0b0011, 0b0100, 0b0101, 0b0110, 0b0111, 0b1000,
            0b0010, 0b0011, 0b0100, 0b0101, 0b0110, 0b0111, 0b1000, 0b1001,
            0b0011, 0b0100, 0b0101, 0b0110, 0b0111, 0b1000, 0b1001, 0b1010,
            0b0100, 0b0101, 0b0110, 0b0111, 0b1000, 0b1001, 0b1010, 0b1011,
            0b0101, 0b0110, 0b0111, 0b1000, 0b1001, 0b1010, 0b1011, 0b1100,
            0b0110, 0b0111, 0b1000, 0b1001, 0b1010, 0b1011, 0b1100, 0b1101,
            0b0111, 0b1000, 0b1001, 0b1010, 0b1011, 0b1100, 0b1101, 0b1110,
            0b1000, 0b1001, 0b1010, 0b1011, 0b1100, 0b1101, 0b1110, 0b1111,
        ]};

        #[rustfmt::skip]
        let colors_flipped_horizontally = [
            0b1000, 0b0111, 0b0110, 0b0101, 0b0100, 0b0011, 0b0010, 0b0001,
            0b1001, 0b1000, 0b0111, 0b0110, 0b0101, 0b0100, 0b0011, 0b0010,
            0b1010, 0b1001, 0b1000, 0b0111, 0b0110, 0b0101, 0b0100, 0b0011,
            0b1011, 0b1010, 0b1001, 0b1000, 0b0111, 0b0110, 0b0101, 0b0100,
            0b1100, 0b1011, 0b1010, 0b1001, 0b1000, 0b0111, 0b0110, 0b0101,
            0b1101, 0b1100, 0b1011, 0b1010, 0b1001, 0b1000, 0b0111, 0b0110,
            0b1110, 0b1101, 0b1100, 0b1011, 0b1010, 0b1001, 0b1000, 0b0111,
            0b1111, 0b1110, 0b1101, 0b1100, 0b1011, 0b1010, 0b1001, 0b1000,
        ];

        #[rustfmt::skip]
        let colors_flipped_vertically = [
            0b1000, 0b1001, 0b1010, 0b1011, 0b1100, 0b1101, 0b1110, 0b1111,
            0b0111, 0b1000, 0b1001, 0b1010, 0b1011, 0b1100, 0b1101, 0b1110,
            0b0110, 0b0111, 0b1000, 0b1001, 0b1010, 0b1011, 0b1100, 0b1101,
            0b0101, 0b0110, 0b0111, 0b1000, 0b1001, 0b1010, 0b1011, 0b1100,
            0b0100, 0b0101, 0b0110, 0b0111, 0b1000, 0b1001, 0b1010, 0b1011,
            0b0011, 0b0100, 0b0101, 0b0110, 0b0111, 0b1000, 0b1001, 0b1010,
            0b0010, 0b0011, 0b0100, 0b0101, 0b0110, 0b0111, 0b1000, 0b1001,
            0b0001, 0b0010, 0b0011, 0b0100, 0b0101, 0b0110, 0b0111, 0b1000,
        ];

        #[rustfmt::skip]
        let colors_flipped_both_axis = [
            0b1111, 0b1110, 0b1101, 0b1100, 0b1011, 0b1010, 0b1001, 0b1000,
            0b1110, 0b1101, 0b1100, 0b1011, 0b1010, 0b1001, 0b1000, 0b0111,
            0b1101, 0b1100, 0b1011, 0b1010, 0b1001, 0b1000, 0b0111, 0b0110,
            0b1100, 0b1011, 0b1010, 0b1001, 0b1000, 0b0111, 0b0110, 0b0101,
            0b1011, 0b1010, 0b1001, 0b1000, 0b0111, 0b0110, 0b0101, 0b0100,
            0b1010, 0b1001, 0b1000, 0b0111, 0b0110, 0b0101, 0b0100, 0b0011,
            0b1001, 0b1000, 0b0111, 0b0110, 0b0101, 0b0100, 0b0011, 0b0010,
            0b1000, 0b0111, 0b0110, 0b0101, 0b0100, 0b0011, 0b0010, 0b0001,
        ];

        assert_eq!(tile_gfx.flip((false, false)), tile_gfx.colors);
        assert_eq!(tile_gfx.flip((true, false)), colors_flipped_horizontally);
        assert_eq!(tile_gfx.flip((false, true)), colors_flipped_vertically);
        assert_eq!(tile_gfx.flip((true, true)), colors_flipped_both_axis);
    }

    /// Load a Gfx composed of many TileGfx in 4bpp format.
    #[test]
    fn load_gfx_from_4bpp() {
        #[rustfmt::skip]
        let expected_gfx = Gfx { tiles: vec![
            TileGfx { colors: [
                0b0001, 0b0010, 0b0011, 0b0100, 0b0101, 0b0110, 0b0111, 0b1000,
                0b0010, 0b0011, 0b0100, 0b0101, 0b0110, 0b0111, 0b1000, 0b1001,
                0b0011, 0b0100, 0b0101, 0b0110, 0b0111, 0b1000, 0b1001, 0b1010,
                0b0100, 0b0101, 0b0110, 0b0111, 0b1000, 0b1001, 0b1010, 0b1011,
                0b0101, 0b0110, 0b0111, 0b1000, 0b1001, 0b1010, 0b1011, 0b1100,
                0b0110, 0b0111, 0b1000, 0b1001, 0b1010, 0b1011, 0b1100, 0b1101,
                0b0111, 0b1000, 0b1001, 0b1010, 0b1011, 0b1100, 0b1101, 0b1110,
                0b1000, 0b1001, 0b1010, 0b1011, 0b1100, 0b1101, 0b1110, 0b1111,
            ]},
            TileGfx { colors: [
                0b1110, 0b1101, 0b1100, 0b1011, 0b1010, 0b1001, 0b1000, 0b0111,
                0b1101, 0b1100, 0b1011, 0b1010, 0b1001, 0b1000, 0b0111, 0b0110,
                0b1100, 0b1011, 0b1010, 0b1001, 0b1000, 0b0111, 0b0110, 0b0101,
                0b1011, 0b1010, 0b1001, 0b1000, 0b0111, 0b0110, 0b0101, 0b0100,
                0b1010, 0b1001, 0b1000, 0b0111, 0b0110, 0b0101, 0b0100, 0b0011,
                0b1001, 0b1000, 0b0111, 0b0110, 0b0101, 0b0100, 0b0011, 0b0010,
                0b1000, 0b0111, 0b0110, 0b0101, 0b0100, 0b0011, 0b0010, 0b0001,
                0b0111, 0b0110, 0b0101, 0b0100, 0b0011, 0b0010, 0b0001, 0b0000,
            ]},
        ]};

        #[rustfmt::skip]
        let gfx_in_4bpp = [
            // TileGfx 0
            0b1010_1010, 0b0110_0110, // Row 0 (bits 1-2)
            0b0101_0101, 0b1100_1100, // Row 1 (bits 1-2)
            0b1010_1010, 0b1001_1001, // Row 2 (bits 1-2)
            0b0101_0101, 0b0011_0011, // Row 3 (bits 1-2)
            0b1010_1010, 0b0110_0110, // Row 4 (bits 1-2)
            0b0101_0101, 0b1100_1100, // Row 5 (bits 1-2)
            0b1010_1010, 0b1001_1001, // Row 6 (bits 1-2)
            0b0101_0101, 0b0011_0011, // Row 7 (bits 1-2)
            0b0001_1110, 0b0000_0001, // Row 0 (bits 3-4)
            0b0011_1100, 0b0000_0011, // Row 1 (bits 3-4)
            0b0111_1000, 0b0000_0111, // Row 2 (bits 3-4)
            0b1111_0000, 0b0000_1111, // Row 3 (bits 3-4)
            0b1110_0001, 0b0001_1111, // Row 4 (bits 3-4)
            0b1100_0011, 0b0011_1111, // Row 5 (bits 3-4)
            0b1000_0111, 0b0111_1111, // Row 6 (bits 3-4)
            0b0000_1111, 0b1111_1111, // Row 7 (bits 3-4)
            // TileGfx 1
            0b0101_0101, 0b1001_1001, // Row 0 (bits 1-2)
            0b1010_1010, 0b0011_0011, // Row 1 (bits 1-2)
            0b0101_0101, 0b0110_0110, // Row 2 (bits 1-2)
            0b1010_1010, 0b1100_1100, // Row 3 (bits 1-2)
            0b0101_0101, 0b1001_1001, // Row 4 (bits 1-2)
            0b1010_1010, 0b0011_0011, // Row 5 (bits 1-2)
            0b0101_0101, 0b0110_0110, // Row 6 (bits 1-2)
            0b1010_1010, 0b1100_1100, // Row 7 (bits 1-2)
            0b1110_0001, 0b1111_1110, // Row 0 (bits 3-4)
            0b1100_0011, 0b1111_1100, // Row 1 (bits 3-4)
            0b1000_0111, 0b1111_1000, // Row 2 (bits 3-4)
            0b0000_1111, 0b1111_0000, // Row 3 (bits 3-4)
            0b0001_1110, 0b1110_0000, // Row 4 (bits 3-4)
            0b0011_1100, 0b1100_0000, // Row 5 (bits 3-4)
            0b0111_1000, 0b1000_0000, // Row 6 (bits 3-4)
            0b1111_0000, 0b0000_0000, // Row 7 (bits 3-4)
        ];

        assert_eq!(from_4bpp(&gfx_in_4bpp), expected_gfx);
    }

    /// Convert a Gfx into a vector with all indexed colors, row by row.
    #[test]
    fn convert_gfx_to_indexed_colors() {
        #[rustfmt::skip]
        let expected_indexed_colors = [
            // Gfx first row of TileGfxs.
            // Gfx row 0.
            [[0b0001, 0b0010, 0b0011, 0b0100, 0b0101, 0b0110, 0b0111, 0b1000]; GFX_TILE_WIDTH / 2].concat(), // color          - row 0
            [[0b1110, 0b1101, 0b1100, 0b1011, 0b1010, 0b1001, 0b1000, 0b0111]; GFX_TILE_WIDTH / 2].concat(), // inverted color - row 0
            // Gfx row 1.
            [[0b0010, 0b0011, 0b0100, 0b0101, 0b0110, 0b0111, 0b1000, 0b1001]; GFX_TILE_WIDTH / 2].concat(), // color          - row 1
            [[0b1101, 0b1100, 0b1011, 0b1010, 0b1001, 0b1000, 0b0111, 0b0110]; GFX_TILE_WIDTH / 2].concat(), // inverted color - row 1
            // Gfx row 2.
            [[0b0011, 0b0100, 0b0101, 0b0110, 0b0111, 0b1000, 0b1001, 0b1010]; GFX_TILE_WIDTH / 2].concat(), // color          - row 2
            [[0b1100, 0b1011, 0b1010, 0b1001, 0b1000, 0b0111, 0b0110, 0b0101]; GFX_TILE_WIDTH / 2].concat(), // inverted color - row 2
            // Gfx row 3.
            [[0b0100, 0b0101, 0b0110, 0b0111, 0b1000, 0b1001, 0b1010, 0b1011]; GFX_TILE_WIDTH / 2].concat(), // color          - row 3
            [[0b1011, 0b1010, 0b1001, 0b1000, 0b0111, 0b0110, 0b0101, 0b0100]; GFX_TILE_WIDTH / 2].concat(), // inverted color - row 3
            // Gfx row 4.
            [[0b0101, 0b0110, 0b0111, 0b1000, 0b1001, 0b1010, 0b1011, 0b1100]; GFX_TILE_WIDTH / 2].concat(), // color          - row 4
            [[0b1010, 0b1001, 0b1000, 0b0111, 0b0110, 0b0101, 0b0100, 0b0011]; GFX_TILE_WIDTH / 2].concat(), // inverted color - row 4
            // Gfx row 5.
            [[0b0110, 0b0111, 0b1000, 0b1001, 0b1010, 0b1011, 0b1100, 0b1101]; GFX_TILE_WIDTH / 2].concat(), // color          - row 5
            [[0b1001, 0b1000, 0b0111, 0b0110, 0b0101, 0b0100, 0b0011, 0b0010]; GFX_TILE_WIDTH / 2].concat(), // inverted color - row 5
            // Gfx row 6.
            [[0b0111, 0b1000, 0b1001, 0b1010, 0b1011, 0b1100, 0b1101, 0b1110]; GFX_TILE_WIDTH / 2].concat(), // color          - row 6
            [[0b1000, 0b0111, 0b0110, 0b0101, 0b0100, 0b0011, 0b0010, 0b0001]; GFX_TILE_WIDTH / 2].concat(), // inverted color - row 6
            // Gfx row 7.
            [[0b1000, 0b1001, 0b1010, 0b1011, 0b1100, 0b1101, 0b1110, 0b1111]; GFX_TILE_WIDTH / 2].concat(), // color          - row 7
            [[0b0111, 0b0110, 0b0101, 0b0100, 0b0011, 0b0010, 0b0001, 0b0000]; GFX_TILE_WIDTH / 2].concat(), // inverted color - row 7
            // Gfx second row of TileGfxs.
            // Gfx row 0.
            [[0b1110, 0b1101, 0b1100, 0b1011, 0b1010, 0b1001, 0b1000, 0b0111]; GFX_TILE_WIDTH / 2].concat(), // inverted color - row 0
            [[0b0001, 0b0010, 0b0011, 0b0100, 0b0101, 0b0110, 0b0111, 0b1000]; GFX_TILE_WIDTH / 2].concat(), // color          - row 0
            // Gfx row 1.
            [[0b1101, 0b1100, 0b1011, 0b1010, 0b1001, 0b1000, 0b0111, 0b0110]; GFX_TILE_WIDTH / 2].concat(), // inverted color - row 1
            [[0b0010, 0b0011, 0b0100, 0b0101, 0b0110, 0b0111, 0b1000, 0b1001]; GFX_TILE_WIDTH / 2].concat(), // color          - row 1
            // Gfx row 2.
            [[0b1100, 0b1011, 0b1010, 0b1001, 0b1000, 0b0111, 0b0110, 0b0101]; GFX_TILE_WIDTH / 2].concat(), // inverted color - row 2
            [[0b0011, 0b0100, 0b0101, 0b0110, 0b0111, 0b1000, 0b1001, 0b1010]; GFX_TILE_WIDTH / 2].concat(), // color          - row 2
            // Gfx row 3.
            [[0b1011, 0b1010, 0b1001, 0b1000, 0b0111, 0b0110, 0b0101, 0b0100]; GFX_TILE_WIDTH / 2].concat(), // inverted color - row 3
            [[0b0100, 0b0101, 0b0110, 0b0111, 0b1000, 0b1001, 0b1010, 0b1011]; GFX_TILE_WIDTH / 2].concat(), // color          - row 3
            // Gfx row 4.
            [[0b1010, 0b1001, 0b1000, 0b0111, 0b0110, 0b0101, 0b0100, 0b0011]; GFX_TILE_WIDTH / 2].concat(), // inverted color - row 4
            [[0b0101, 0b0110, 0b0111, 0b1000, 0b1001, 0b1010, 0b1011, 0b1100]; GFX_TILE_WIDTH / 2].concat(), // color          - row 4
            // Gfx row 5.
            [[0b1001, 0b1000, 0b0111, 0b0110, 0b0101, 0b0100, 0b0011, 0b0010]; GFX_TILE_WIDTH / 2].concat(), // inverted color - row 5
            [[0b0110, 0b0111, 0b1000, 0b1001, 0b1010, 0b1011, 0b1100, 0b1101]; GFX_TILE_WIDTH / 2].concat(), // color          - row 5
            // Gfx row 6.
            [[0b1000, 0b0111, 0b0110, 0b0101, 0b0100, 0b0011, 0b0010, 0b0001]; GFX_TILE_WIDTH / 2].concat(), // inverted color - row 6
            [[0b0111, 0b1000, 0b1001, 0b1010, 0b1011, 0b1100, 0b1101, 0b1110]; GFX_TILE_WIDTH / 2].concat(), // color          - row 6
            // Gfx row 7.
            [[0b0111, 0b0110, 0b0101, 0b0100, 0b0011, 0b0010, 0b0001, 0b0000]; GFX_TILE_WIDTH / 2].concat(), // inverted color - row 7
            [[0b1000, 0b1001, 0b1010, 0b1011, 0b1100, 0b1101, 0b1110, 0b1111]; GFX_TILE_WIDTH / 2].concat(), // color          - row 7
        ]
        .concat();

        #[rustfmt::skip]
        let half_size_tile_gfx = vec![TileGfx { colors: [
            0b0001, 0b0010, 0b0011, 0b0100, 0b0101, 0b0110, 0b0111, 0b1000,
            0b0010, 0b0011, 0b0100, 0b0101, 0b0110, 0b0111, 0b1000, 0b1001,
            0b0011, 0b0100, 0b0101, 0b0110, 0b0111, 0b1000, 0b1001, 0b1010,
            0b0100, 0b0101, 0b0110, 0b0111, 0b1000, 0b1001, 0b1010, 0b1011,
            0b0101, 0b0110, 0b0111, 0b1000, 0b1001, 0b1010, 0b1011, 0b1100,
            0b0110, 0b0111, 0b1000, 0b1001, 0b1010, 0b1011, 0b1100, 0b1101,
            0b0111, 0b1000, 0b1001, 0b1010, 0b1011, 0b1100, 0b1101, 0b1110,
            0b1000, 0b1001, 0b1010, 0b1011, 0b1100, 0b1101, 0b1110, 0b1111,
        ]}; GFX_TILE_WIDTH / 2];

        #[rustfmt::skip]
        let half_size_inverted_tile_gfx = vec![TileGfx { colors: [
            0b1110, 0b1101, 0b1100, 0b1011, 0b1010, 0b1001, 0b1000, 0b0111,
            0b1101, 0b1100, 0b1011, 0b1010, 0b1001, 0b1000, 0b0111, 0b0110,
            0b1100, 0b1011, 0b1010, 0b1001, 0b1000, 0b0111, 0b0110, 0b0101,
            0b1011, 0b1010, 0b1001, 0b1000, 0b0111, 0b0110, 0b0101, 0b0100,
            0b1010, 0b1001, 0b1000, 0b0111, 0b0110, 0b0101, 0b0100, 0b0011,
            0b1001, 0b1000, 0b0111, 0b0110, 0b0101, 0b0100, 0b0011, 0b0010,
            0b1000, 0b0111, 0b0110, 0b0101, 0b0100, 0b0011, 0b0010, 0b0001,
            0b0111, 0b0110, 0b0101, 0b0100, 0b0011, 0b0010, 0b0001, 0b0000,
        ]}; GFX_TILE_WIDTH / 2];

        let gfx = Gfx {
            tiles: [
                half_size_tile_gfx.clone(),
                half_size_inverted_tile_gfx.clone(),
                half_size_inverted_tile_gfx.clone(),
                half_size_tile_gfx.clone(),
            ]
            .concat(),
        };

        assert_eq!(gfx.to_indexed_colors(), expected_indexed_colors);
    }
}
