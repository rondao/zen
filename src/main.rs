use std::{error::Error, fs, usize};

use image::{Rgb, RgbImage};
use zen::{
    compress,
    graphics::{gfx, palette, Rgb888},
};

fn main() -> Result<(), Box<dyn Error>> {
    let palette_compressed = fs::read("/home/rondao/dev/rust/snes_data/Crateria.tpl.bin")?;
    assert_eq!(
        compress::decompress_lz5(&palette_compressed)?,
        fs::read("/home/rondao/dev/rust/snes_data/Crateria.tpl")?
    );
    let gfx_compressed = fs::read("/home/rondao/dev/rust/snes_data/Crateria.gfx.bin")?;
    assert_eq!(
        compress::decompress_lz5(&gfx_compressed)?,
        fs::read("/home/rondao/dev/rust/snes_data/Crateria.gfx")?
    );
    assert_eq!(
        compress::decompress_lz5(&fs::read("/home/rondao/dev/rust/snes_data/CRE.gfx.bin")?)?,
        fs::read("/home/rondao/dev/rust/snes_data/CRE.gfx")?
    );
    assert_eq!(
        compress::decompress_lz5(&fs::read(
            "/home/rondao/dev/rust/snes_data/Crateria.level.bin"
        )?)?,
        fs::read("/home/rondao/dev/rust/snes_data/Crateria.level")?
    );

    let palette = palette::from_bytes(&compress::decompress_lz5(&palette_compressed)?)?;
    let mut palette_colors = palette.to_colors().into_iter();

    let mut img: RgbImage = RgbImage::new(16, 16);
    for y in 0..16 {
        for x in 0..16 {
            let color = palette_colors.next().unwrap();
            img.put_pixel(x, y, Rgb([color.r, color.g, color.b]));
        }
    }
    img.save("/home/rondao/palette.png")?;

    let gfx = gfx::from_4bpp(&compress::decompress_lz5(&gfx_compressed)?);

    let mut img: RgbImage = RgbImage::new(16 * 8, gfx.tiles.len() as u32);
    for (tile_num, tile) in gfx.tiles.iter().enumerate() {
        // Position of the Tile8 we are drawing.
        let tx = (tile_num % 16) * 8;
        let ty = (tile_num / 16) * 8;

        // Drawing the 8 * 8 pixels of a Tile8.
        for tpx in 0..8 {
            for tpy in 0..8 {
                let idx_color = tile.colors[tpx + tpy * 8] as usize;
                let color: Rgb888 = palette.sub_palettes[4].colors[idx_color].into();

                img.put_pixel(
                    (tx + tpx) as u32,
                    (ty + tpy) as u32,
                    Rgb([color.r, color.g, color.b]),
                );
            }
        }
    }
    img.save("/home/rondao/gfx.png")?;

    Ok(())
}
