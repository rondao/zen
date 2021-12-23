use std::error::Error;

use zen::super_metroid;

fn main() -> Result<(), Box<dyn Error>> {
    let sm = super_metroid::load_unheadered_rom(
        "/home/rondao/dev/snes_data/Super Metroid (JU) [!].smc",
    )?;

    if let Some(image) = sm.room_to_image(0x8F96BA, 2, 1, 3, 1) {
        image.save("/home/rondao/dev/snes_data/0x8F96BA.png")?;
    }

    Ok(())
}
