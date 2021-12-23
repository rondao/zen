use std::error::Error;

use zen::{
    graphics::gfx::{Gfx, Tile8},
    super_metroid::rom,
};

fn main() -> Result<(), Box<dyn Error>> {
    let rom = rom::load_unheadered_rom("/home/rondao/dev/snes_data/Super Metroid (JU) [!].smc")?;

    // Gfx + 64 Empty + CRE
    let gfx_cre = Gfx {
        tiles: [
            &rom.graphics[4].tiles[..],
            &[Tile8 { colors: [0; 64] }; 64],
            &rom.cre_gfx.tiles[..],
        ]
        .concat(),
    };

    // CRE + Crateria Tileset
    let tileset_cre = [&rom.cre_tileset[..], &rom.tilesets[4]].concat();

    rom.levels
        .get_level(0xC6DEE0)
        .to_image(&tileset_cre, &rom.palettes[7], &gfx_cre)
        .save("/home/rondao/dev/snes_data/Kraid_Lair_level.png")?;

    Ok(())
}
