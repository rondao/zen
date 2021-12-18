use image::{Rgb, RgbImage};

use crate::graphics::{gfx::Gfx, Palette, Rgb888};

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

impl Gfx {
    pub fn to_image(&self, palette: Palette, sub_palette: usize) -> RgbImage {
        let mut img: RgbImage = RgbImage::new(16 * 8, self.tiles.len() as u32);
        for (tile_num, tile) in self.tiles.iter().enumerate() {
            // Position of the Tile8 we are drawing.
            let tx = (tile_num % 16) * 8;
            let ty = (tile_num / 16) * 8;

            // Drawing the 8 * 8 pixels of a Tile8.
            for tpx in 0..8 {
                for tpy in 0..8 {
                    let idx_color = tile.colors[tpx + tpy * 8] as usize;
                    let color: Rgb888 = palette.sub_palettes[sub_palette].colors[idx_color].into();

                    img.put_pixel(
                        (tx + tpx) as u32,
                        (ty + tpy) as u32,
                        Rgb([color.r, color.g, color.b]),
                    );
                }
            }
        }
        img
    }
}
