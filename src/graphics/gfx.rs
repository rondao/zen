use std::convert::TryInto;

pub const TILE_SIZE: usize = 8;
pub const GFX_TILE_WIDTH: usize = 16;

#[derive(Debug, Default, Clone)]
pub struct Gfx {
    pub tiles: Vec<Tile8>,
}

#[derive(Debug, Clone, Copy)]
pub struct Tile8 {
    pub colors: [u8; TILE_SIZE * TILE_SIZE],
}

impl Tile8 {
    /// Reference: https://sneslab.net/wiki/Graphics_Format
    ///  [r0, bp1], [r0, bp2], [r1, bp1], [r1, bp2], [r2, bp1], [r2, bp2], [r3, bp1], [r3, bp2]
    ///  [r4, bp1], [r4, bp2], [r5, bp1], [r5, bp2], [r6, bp1], [r6, bp2], [r7, bp1], [r7, bp2]
    ///  [r0, bp3], [r0, bp4], [r1, bp3], [r1, bp4], [r2, bp3], [r2, bp4], [r3, bp3], [r3, bp4]
    ///  [r4, bp3], [r4, bp4], [r5, bp3], [r5, bp4], [r6, bp3], [r6, bp4], [r7, bp3], [r7, bp4]
    fn tile_4bpp(source: &[u8]) -> Tile8 {
        let mut colors: Vec<u8> = Vec::with_capacity(TILE_SIZE * TILE_SIZE);

        // One half of the data has bits 0,1 and the other has bits 2,3 to compose one color.
        let (bitplane_01, bitplane_23) = source.split_at(source.len() / 2);
        // First half's first and second byte has 0 and 1 bits respectively.
        // Second half's first and second byte has 2 and 3 bits respectively.
        for (pixel_01, pixel_23) in bitplane_01.chunks(2).zip(bitplane_23.chunks(2)) {
            // Each 'i' bit of the 4 bytes collected at each iteration composes one color.
            //  [bp1, bp2, bp3, bp4, bp5, bp6, bp7, bp8]
            for i in (0..TILE_SIZE).rev() {
                colors.push(
                    (((pixel_01[0] >> i) & 1) << 0)
                        + (((pixel_01[1] >> i) & 1) << 1)
                        + (((pixel_23[0] >> i) & 1) << 2)
                        + (((pixel_23[1] >> i) & 1) << 3),
                )
            }
        }
        Tile8 {
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
        tiles: source.chunks(TILE_SIZE * 4).map(Tile8::tile_4bpp).collect(),
    }
}

impl Gfx {
    pub fn to_colors(&self) -> Vec<u8> {
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
}
