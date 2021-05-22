use std::{error::Error, fs};

use image::{Rgb, RgbImage};
use zen::graphics::Palette;

fn main() -> Result<(), Box<dyn Error>> {
    let palette_bytes = fs::read("/home/rondao/dev/rust/snes_data/Crateria.tpl")?;
    let palette = Palette::from_bytes(&palette_bytes);
    let mut palette_colors = palette.to_colors().into_iter();

    let mut img: RgbImage = RgbImage::new(16, 8);
    for y in 0..8 {
        for x in 0..16 {
            let color = palette_colors.next().unwrap();
            img.put_pixel(x, y, Rgb([color.r, color.g, color.b]));
        }
    }
    img.save("/home/rondao/palette.png")?;

    Ok(())
}
