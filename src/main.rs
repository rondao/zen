use std::{error::Error, fs};

use ::image::RgbImage;
use zen::{
    compress,
    graphics::{gfx, palette},
};

fn main() -> Result<(), Box<dyn Error>> {
    let palette_compressed = fs::read("/home/rondao/dev/snes_data/Crateria.tpl.bin")?;
    assert_eq!(
        compress::decompress_lz5(&palette_compressed)?,
        fs::read("/home/rondao/dev/snes_data/Crateria.tpl")?
    );
    let gfx_compressed = fs::read("/home/rondao/dev/snes_data/Crateria.gfx.bin")?;
    assert_eq!(
        compress::decompress_lz5(&gfx_compressed)?,
        fs::read("/home/rondao/dev/snes_data/Crateria.gfx")?
    );
    let cre_compressed = fs::read("/home/rondao/dev/snes_data/CRE.gfx.bin")?;
    assert_eq!(
        compress::decompress_lz5(&cre_compressed)?,
        fs::read("/home/rondao/dev/snes_data/CRE.gfx")?
    );
    assert_eq!(
        compress::decompress_lz5(&fs::read("/home/rondao/dev/snes_data/Crateria.level.bin")?)?,
        fs::read("/home/rondao/dev/snes_data/Crateria.level")?
    );

    let palette = palette::from_bytes(&compress::decompress_lz5(&palette_compressed)?)?;
    RgbImage::from(&palette).save("/home/rondao/dev/snes_data/palette.png")?;

    let gfx = gfx::from_4bpp(&compress::decompress_lz5(&gfx_compressed)?);
    gfx.to_image(&palette, 4)
        .save("/home/rondao/dev/snes_data/gfx.png")?;

    let cre = gfx::from_4bpp(&compress::decompress_lz5(&cre_compressed)?);
    cre.to_image(&palette, 0)
        .save("/home/rondao/dev/snes_data/cre.png")?;

    Ok(())
}
