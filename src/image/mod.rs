use image::{Rgb, RgbImage};

use crate::{
    graphics::{
        gfx::{Gfx, Tile8},
        Palette, Rgb888,
    },
    super_metroid::{room::Room, tileset::Tileset},
};

impl From<&Palette> for RgbImage {
    fn from(item: &Palette) -> Self {
        let mut palette_colors = item.to_colors().into_iter();

        let mut img: RgbImage = RgbImage::new(16, 16);
        for y in 0..16 {
            for x in 0..16 {
                let color = palette_colors.next().unwrap();
                img.put_pixel(x, y, Rgb([color.r, color.g, color.b]));
            }
        }
        img
    }
}

impl Tile8 {
    pub fn draw(
        &self,
        image: &mut RgbImage,
        origin: (usize, usize),
        flip: (bool, bool),
        palette: &Palette,
        sub_palette: usize,
    ) {
        for tpx in 0..8 {
            for tpy in 0..8 {
                let idx_color = self.colors[tpx + tpy * 8] as usize;
                let color: Rgb888 = palette.sub_palettes[sub_palette].colors[idx_color].into();

                image.put_pixel(
                    (origin.0 + if flip.0 { 7 - tpx } else { tpx }) as u32,
                    (origin.1 + if flip.1 { 7 - tpy } else { tpy }) as u32,
                    Rgb([color.r, color.g, color.b]),
                );
            }
        }
    }
}

impl Gfx {
    pub fn to_image(&self, palette: &Palette, sub_palette: usize) -> RgbImage {
        let mut img: RgbImage = RgbImage::new(16 * 8, self.tiles.len() as u32);
        for (tile_num, tile) in self.tiles.iter().enumerate() {
            // Position of the Tile8 we are drawing.
            let tx = (tile_num % 16) * 8;
            let ty = (tile_num / 16) * 8;

            tile.draw(&mut img, (tx, ty), (false, false), palette, sub_palette);
        }
        img
    }
}

pub fn tileset_to_image(tileset: &Tileset, palette: &Palette, graphics: &Gfx) -> RgbImage {
    let mut img: RgbImage = RgbImage::new(16 * 32, 16 * 32);

    let mut tiles = tileset.iter();
    for ty in 0..32 {
        for tx in 0..32 {
            // Each tile is composed of 4 smaller 'tile8'.
            for t in 0..4 {
                let tile = tiles.next().unwrap();
                let tile8 = &graphics.tiles[tile.gfx_index as usize];
                tile8.draw(
                    &mut img,
                    (tx * 16 + (t % 2) * 8, ty * 16 + (t / 2) * 8),
                    (tile.x_flip, tile.y_flip),
                    palette,
                    tile.sub_palette as usize,
                );
            }
        }
    }
    img
}

impl Room {
    pub fn to_image(&self, tileset: &Tileset, palette: &Palette, graphics: &Gfx) -> RgbImage {
        let mut img: RgbImage = RgbImage::new(9 * 16 * 16, 5 * 16 * 16);
        for (index, block) in self.layer1.iter().enumerate() {
            let tileset_tile = block.block_number as usize * 4;
            let mut tiles: Vec<_> = tileset[tileset_tile..tileset_tile + 4]
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

            // Each block is composed of 4 smaller 'tile8'.
            for t in 0..4 {
                let tile = tiles[t];
                let tile8 = &graphics.tiles[tile.gfx_index as usize];
                tile8.draw(
                    &mut img,
                    (
                        (index % (9 * 16)) * 16 + (t % 2) * 8,
                        (index / (9 * 16)) * 16 + (t / 2) * 8,
                    ),
                    (tile.x_flip ^ block.x_flip, tile.y_flip ^ block.y_flip),
                    palette,
                    tile.sub_palette as usize,
                );
            }
        }
        img
    }
}
