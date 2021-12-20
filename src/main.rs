use std::{error::Error, fs};

use ::image::RgbImage;
use zen::{
    address::LoRom,
    compress,
    graphics::{
        gfx::{self, Gfx, Tile8},
        palette,
    },
    image::tileset_to_image,
    super_metroid::{room, tileset},
    Rom,
};

fn main() -> Result<(), Box<dyn Error>> {
    // Rom
    let rom = Rom {
        rom: fs::read("/home/rondao/dev/snes_data/Super Metroid (JU) [!].smc")?,
    };

    // Palette
    let palette_compressed = rom.read(
        LoRom { address: 0xC2_AD7C }.into(),
        LoRom { address: 0xC2_AE5D }.into(),
    );
    assert_eq!(
        compress::decompress_lz5(palette_compressed)?,
        fs::read("/home/rondao/dev/snes_data/Crateria.tpl")?
    );
    let palette = palette::from_bytes(&compress::decompress_lz5(palette_compressed)?)?;
    RgbImage::from(&palette).save("/home/rondao/dev/snes_data/palette.png")?;

    // Gfx
    let gfx_compressed = rom.read(
        LoRom { address: 0xBA_C629 }.into(),
        LoRom { address: 0xBA_F911 }.into(),
    );
    assert_eq!(
        compress::decompress_lz5(gfx_compressed)?,
        fs::read("/home/rondao/dev/snes_data/Crateria.gfx")?
    );
    let gfx = gfx::from_4bpp(&compress::decompress_lz5(gfx_compressed)?);
    gfx.to_image(&palette, 4)
        .save("/home/rondao/dev/snes_data/gfx.png")?;

    // CRE
    let cre_compressed = rom.read(
        LoRom { address: 0xB9_8000 }.into(),
        LoRom { address: 0xB9_A09D }.into(),
    );
    assert_eq!(
        compress::decompress_lz5(cre_compressed)?,
        fs::read("/home/rondao/dev/snes_data/CRE.gfx")?
    );
    let cre = gfx::from_4bpp(&compress::decompress_lz5(cre_compressed)?);
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
    let crateria_tileset_compressed = rom.read(
        LoRom { address: 0xC1_B6F6 }.into(),
        LoRom { address: 0xC1_BEEE }.into(),
    );
    let crateria_tileset =
        tileset::from_bytes(&compress::decompress_lz5(crateria_tileset_compressed)?);

    // CRE Tileset
    let cre_tileset_compressed = rom.read(
        LoRom { address: 0xB9_A09D }.into(),
        LoRom { address: 0xB9_A634 }.into(),
    );
    let cre_tileset = tileset::from_bytes(&compress::decompress_lz5(cre_tileset_compressed)?);

    // CRE + Crateria Tileset
    let cre_crateria_tileset = [&cre_tileset[..], &crateria_tileset[..]].concat();

    tileset_to_image(&cre_crateria_tileset, &palette, &gfx_cre)
        .save("/home/rondao/dev/snes_data/cre_crateria_tileset.png")?;

    // Crateria Room
    let crateria_room_compressed = rom.read(
        LoRom { address: 0xC2_C2BB }.into(),
        LoRom { address: 0xC2_D6E8 }.into(),
    );
    assert_eq!(
        compress::decompress_lz5(crateria_room_compressed)?,
        fs::read("/home/rondao/dev/snes_data/Crateria.room")?
    );

    let crateria_room =
        room::from_bytes(&compress::decompress_lz5(crateria_room_compressed)?, true);
    crateria_room
        .to_image(&cre_crateria_tileset, &palette, &gfx_cre)
        .save("/home/rondao/dev/snes_data/crateria.room.png")?;

    Ok(())
}
