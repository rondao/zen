use std::error::Error;

use zen::super_metroid::rom;

fn main() -> Result<(), Box<dyn Error>> {
    let rom = rom::load_unheadered_rom("/home/rondao/dev/snes_data/Super Metroid (JU) [!].smc")?;

    rom.levels
        .get_level(0xC6DEE0)
        .to_image(
            &rom.tileset_with_cre(4),
            &rom.palettes[7],
            &rom.gfx_with_cre(4),
        )
        .save("/home/rondao/dev/snes_data/Kraid_Lair_level.png")?;

    Ok(())
}
