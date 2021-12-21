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
    super_metroid::{level_data::Levels, room, state::States, tileset},
    Rom,
};

fn main() -> Result<(), Box<dyn Error>> {
    // Rom
    let rom = Rom {
        rom: fs::read("/home/rondao/dev/snes_data/Super Metroid (JU) [!].smc")?,
    };

    // Palette
    let palette_data = compress::decompress_lz5(rom.offset(LoRom { address: 0xC2_AD7C }.into()))?;
    assert_eq!(
        palette_data,
        fs::read("/home/rondao/dev/snes_data/Crateria.tpl")?
    );
    let palette = palette::from_bytes(&palette_data)?;
    RgbImage::from(&palette).save("/home/rondao/dev/snes_data/palette.png")?;

    // Gfx
    let gfx_data = compress::decompress_lz5(rom.offset(LoRom { address: 0xBA_C629 }.into()))?;
    assert_eq!(
        gfx_data,
        fs::read("/home/rondao/dev/snes_data/Crateria.gfx")?
    );
    let gfx = gfx::from_4bpp(&gfx_data);
    gfx.to_image(&palette, 4)
        .save("/home/rondao/dev/snes_data/gfx.png")?;

    // CRE
    let cre_data = compress::decompress_lz5(rom.offset(LoRom { address: 0xB9_8000 }.into()))?;
    assert_eq!(cre_data, fs::read("/home/rondao/dev/snes_data/CRE.gfx")?);

    let cre = gfx::from_4bpp(&cre_data);
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
    let crateria_tileset_data =
        compress::decompress_lz5(rom.offset(LoRom { address: 0xC1_B6F6 }.into()))?;
    let crateria_tileset = tileset::from_bytes(&crateria_tileset_data);

    // CRE Tileset
    let cre_tileset_data =
        compress::decompress_lz5(rom.offset(LoRom { address: 0xB9_A09D }.into()))?;
    let cre_tileset = tileset::from_bytes(&cre_tileset_data);

    // CRE + Crateria Tileset
    let cre_crateria_tileset = [&cre_tileset[..], &crateria_tileset[..]].concat();

    tileset_to_image(&cre_crateria_tileset, &palette, &gfx_cre)
        .save("/home/rondao/dev/snes_data/cre_crateria_tileset.png")?;

    // Crateria Room
    let crateria_room_data = rom.offset(LoRom { address: 0x8F_91F8 }.into());
    let crateria_room = room::from_bytes(0x91F8, &crateria_room_data);

    // States
    let mut states = States::default();

    // Load states from Room
    let state_address = crateria_room.state_conditions[0].state_address;
    states.load_bytes(
        state_address as usize,
        rom.offset(
            LoRom {
                address: 0x8F_0000 + state_address as usize,
            }
            .into(),
        ),
    );
    let state = states.get_state(state_address as usize);

    // Levels
    let mut levels = Levels::default();

    // Load level from State
    levels.load_from_bytes(
        state.level_data as usize,
        &compress::decompress_lz5(
            rom.offset(
                LoRom {
                    address: state.level_data as usize,
                }
                .into(),
            ),
        )?,
        true,
    );
    let crateria_level = levels.get_level(state.level_data as usize);

    crateria_level
        .to_image(&cre_crateria_tileset, &palette, &gfx_cre)
        .save("/home/rondao/dev/snes_data/crateria.level.png")?;

    Ok(())
}
