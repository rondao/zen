use std::error::Error;

use zen::super_metroid;

fn main() -> Result<(), Box<dyn Error>> {
    let mut sm = super_metroid::load_unheadered_rom(
        "/home/rondao/dev/snes_data/test/Super Metroid (JU) [!].smc",
    )?;

    sm.save_to_file("/home/rondao/roms/snes/MyHack.smc")
        .unwrap();

    Ok(())
}
