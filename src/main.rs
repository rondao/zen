use std::error::Error;

use zen::{
    image::tileset_to_image,
    super_metroid::{self, address::ROOMS},
};

fn main() -> Result<(), Box<dyn Error>> {
    let mut sm = super_metroid::load_unheadered_rom(
        "/home/rondao/dev/snes_data/test/Super Metroid (JU) [!].smc",
    )?;

    sm.save_to_file("/home/rondao/roms/snes_data/MyHack.smc")
        .unwrap();

    Ok(())
}
