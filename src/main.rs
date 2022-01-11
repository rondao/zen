use std::error::Error;

use zen::{
    image::tileset_to_image,
    super_metroid::{self, address::ROOMS},
};

fn main() -> Result<(), Box<dyn Error>> {
    let mut sm = super_metroid::load_unheadered_rom(
        "/home/rondao/dev/snes_data/Super Metroid (JU) [!].smc",
    )?;

    sm.save_to_file("/home/rondao/roms/snes/MyHack.smc")
        .unwrap();

    for (address, palette) in sm.palettes.iter() {
        println!("Palette: {:x}", address);
        palette
            .to_image()
            .save(format!("/home/rondao/dev/snes_data/{:x}.png", address))?;
    }

    for (address, gfx) in sm
        .graphics
        .keys()
        .map(|address| (address, sm.gfx_with_cre(*address)))
    {
        println!("GFX: {:x}", *address);
        gfx.to_image(&sm.palettes[&(sm.tilesets[0].palette as usize)], 0)
            .save(format!("/home/rondao/dev/snes_data/{:x}.png", address))?;
    }

    for (i, tileset) in sm.tilesets.iter().enumerate() {
        let image = if tileset.use_cre {
            println!("Tileset with CRE: {}", i);
            tileset_to_image(
                &sm.tile_table_with_cre(tileset.tile_table as usize),
                &sm.palettes[&(tileset.palette as usize)],
                &sm.gfx_with_cre(tileset.graphic as usize),
            )
        } else {
            println!("Tileset: {}", i);
            tileset_to_image(
                &sm.tile_tables[&(tileset.tile_table as usize)],
                &sm.palettes[&(tileset.palette as usize)],
                &sm.graphics[&(tileset.graphic as usize)],
            )
        };
        image.save(format!("/home/rondao/dev/snes_data/tileset_{}.png", i))?;
    }

    for address in ROOMS {
        println!("Room: {:x}", *address);
        if let Some(image) = sm.room_to_image(*address, 0) {
            image.save(format!("/home/rondao/dev/snes_data/{:x}.png", *address))?;
        }
    }

    Ok(())
}
