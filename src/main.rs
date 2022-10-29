use std::{error::Error, fs};

use zen::super_metroid;

fn main() -> Result<(), Box<dyn Error>> {
    let mut sm = super_metroid::load_unheadered_rom(fs::read(
        "/home/rondao/dev/snes_data/test/Super Metroid (JU) [!].smc",
    )?)?;

    sm.save_to_rom();
    sm.save_to_file("/home/rondao/roms/snes/MyHack.smc")
        .unwrap();

    Ok(())
}
