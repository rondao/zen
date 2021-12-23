use std::error::Error;

use zen::super_metroid;

fn main() -> Result<(), Box<dyn Error>> {
    let sm = super_metroid::load_unheadered_rom(
        "/home/rondao/dev/snes_data/Super Metroid (JU) [!].smc",
    )?;

    sm.levels
        .get_level(0xC6DEE0)
        .to_image(
            &sm.tileset_with_cre(4),
            &sm.palettes[7],
            &sm.gfx_with_cre(4),
        )
        .save("/home/rondao/dev/snes_data/Kraid_Lair_level.png")?;

    Ok(())
}
