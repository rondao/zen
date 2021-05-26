use std::{error::Error, fs};

use image::{Rgb, RgbImage};
use zen::{compress, graphics::palette};

fn main() -> Result<(), Box<dyn Error>> {
    let palette_compressed = fs::read("/home/rondao/dev/rust/snes_data/Crateria.tpl.bin")?;
    let palette_decompressed = fs::read("/home/rondao/dev/rust/snes_data/Crateria.tpl")?;
    assert_eq!(
        compress::decompress_lz5(&palette_compressed),
        palette_decompressed
    );
    assert_eq!(
        compress::decompress_lz5(&fs::read(
            "/home/rondao/dev/rust/snes_data/Crateria.gfx.bin"
        )?),
        fs::read("/home/rondao/dev/rust/snes_data/Crateria.gfx")?
    );
    assert_eq!(
        compress::decompress_lz5(&fs::read("/home/rondao/dev/rust/snes_data/CRE.gfx.bin")?),
        fs::read("/home/rondao/dev/rust/snes_data/CRE.gfx")?
    );
    assert_eq!(
        compress::decompress_lz5(&fs::read(
            "/home/rondao/dev/rust/snes_data/Crateria.level.bin"
        )?),
        fs::read("/home/rondao/dev/rust/snes_data/Crateria.level")?
    );

    let palette = palette::from_bytes(&palette_decompressed)?;
    let mut palette_colors = palette.to_colors().into_iter();

    let mut img: RgbImage = RgbImage::new(16, 16);
    for y in 0..16 {
        for x in 0..16 {
            let color = palette_colors.next().unwrap();
            img.put_pixel(x, y, Rgb([color.r, color.g, color.b]));
        }
    }
    img.save("/home/rondao/palette.png")?;

    Ok(())
}
