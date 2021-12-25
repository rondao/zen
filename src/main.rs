use std::error::Error;

use zen::super_metroid::{self, address::ROOMS};

fn main() -> Result<(), Box<dyn Error>> {
    let sm = super_metroid::load_unheadered_rom(
        "/home/rondao/dev/snes_data/Super Metroid (JU) [!].smc",
    )?;

    for address in ROOMS {
        println!("Room: {:x}", *address);
        if let Some(image) = sm.room_to_image(*address, 0) {
            image.save(format!("/home/rondao/dev/snes_data/{:x}.png", *address))?;
        }
    }

    Ok(())
}
