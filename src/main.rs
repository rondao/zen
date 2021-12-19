use std::{error::Error, fs};

use ::image::RgbImage;
use zen::{
    compress,
    graphics::{
        gfx::{self, Gfx, Tile8},
        palette,
    },
    image::tileset_to_image,
    super_metroid::tileset,
};

fn main() -> Result<(), Box<dyn Error>> {
    // Palette
    let palette_compressed = fs::read("/home/rondao/dev/snes_data/Crateria.tpl.bin")?;
    assert_eq!(
        compress::decompress_lz5(&palette_compressed)?,
        fs::read("/home/rondao/dev/snes_data/Crateria.tpl")?
    );
    let palette = palette::from_bytes(&compress::decompress_lz5(&palette_compressed)?)?;
    RgbImage::from(&palette).save("/home/rondao/dev/snes_data/palette.png")?;

    // Gfx
    let gfx_compressed = fs::read("/home/rondao/dev/snes_data/Crateria.gfx.bin")?;
    assert_eq!(
        compress::decompress_lz5(&gfx_compressed)?,
        fs::read("/home/rondao/dev/snes_data/Crateria.gfx")?
    );
    let gfx = gfx::from_4bpp(&compress::decompress_lz5(&gfx_compressed)?);
    gfx.to_image(&palette, 4)
        .save("/home/rondao/dev/snes_data/gfx.png")?;

    // CRE
    let cre_compressed = fs::read("/home/rondao/dev/snes_data/CRE.gfx.bin")?;
    assert_eq!(
        compress::decompress_lz5(&cre_compressed)?,
        fs::read("/home/rondao/dev/snes_data/CRE.gfx")?
    );
    let cre = gfx::from_4bpp(&compress::decompress_lz5(&cre_compressed)?);
    cre.to_image(&palette, 0)
        .save("/home/rondao/dev/snes_data/cre.png")?;

    // Gfx + 64 Empty + CRE
    let gfx_cre = Gfx {
        tiles: [
            &gfx.tiles[..],
            &[Tile8 { colors: [0; 64] }; 64],
            &cre.tiles[..],
        ]
        .concat(),
    };
    gfx_cre
        .to_image(&palette, 0)
        .save("/home/rondao/dev/snes_data/gfx_cre.png")?;

    // Crateria Tileset
    let crateria_tileset_compressed = fs::read("/home/rondao/dev/snes_data/Crateria.tls.bin")?;
    let crateria_tileset =
        tileset::from_bytes(&compress::decompress_lz5(&crateria_tileset_compressed)?);

    // CRE Tileset
    let cre_tileset_compressed = fs::read("/home/rondao/dev/snes_data/CRE.tls.bin")?;
    let cre_tileset = tileset::from_bytes(&compress::decompress_lz5(&cre_tileset_compressed)?);

    // CRE + Crateria Tileset
    let cre_crateria_tileset = [&cre_tileset[..], &crateria_tileset[..]].concat();

    tileset_to_image(&cre_crateria_tileset, &palette, &gfx_cre)
        .save("/home/rondao/dev/snes_data/cre_crateria_tileset.png")?;

    Ok(())
}
