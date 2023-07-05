use std::fs;

use zen::{
    image::tileset_to_image,
    super_metroid::{self},
};

/// Convert Palettes to image after saving palettes to rom.
#[test]
#[ignore = "Requires ROM data to run, which is Copyrighted."]
fn convert_super_metroid_palettes_to_image() {
    let mut sm = super_metroid::load_unheadered_rom(
        fs::read("/home/rondao/dev/snes_data/test/Super Metroid (JU) [!].smc").unwrap(),
    )
    .unwrap();

    let remapped_palettes = sm.save_palettes_to_rom();

    for (old_address, new_address) in remapped_palettes {
        let expected_image = image::open(format!(
            "/home/rondao/dev/snes_data/test/{:x}.png",
            old_address
        ))
        .unwrap();
        assert_eq!(
            &sm.palettes[&new_address].to_image(),
            expected_image.as_rgb8().unwrap()
        );
    }
}

/// Convert Gfx to image.
#[test]
#[ignore = "Requires ROM data to run, which is Copyrighted."]
fn convert_super_metroid_gfxs_to_image() {
    let sm = super_metroid::load_unheadered_rom(
        fs::read("/home/rondao/dev/snes_data/test/Super Metroid (JU) [!].smc").unwrap(),
    )
    .unwrap();

    for (address, gfx) in sm
        .graphics
        .keys()
        .map(|address| (address, sm.gfx_with_cre(*address)))
    {
        let expected_image =
            image::open(format!("/home/rondao/dev/snes_data/test/{:x}.png", address)).unwrap();
        assert_eq!(
            &gfx.to_image(&sm.palettes[&(sm.tilesets[0].palette as usize)], 0),
            expected_image.as_rgb8().unwrap()
        );
    }
}

/// Convert Tileset to image.
#[test]
#[ignore = "Requires ROM data to run, which is Copyrighted."]
fn convert_super_metroid_tilesets_to_image() {
    let sm = super_metroid::load_unheadered_rom(
        fs::read("/home/rondao/dev/snes_data/test/Super Metroid (JU) [!].smc").unwrap(),
    )
    .unwrap();

    for (i, tileset) in sm.tilesets.iter().enumerate() {
        let expected_image =
            image::open(format!("/home/rondao/dev/snes_data/test/tileset_{}.png", i)).unwrap();
        if tileset.use_cre {
            assert_eq!(
                &tileset_to_image(
                    &sm.tile_table_with_cre(tileset.tile_table as usize),
                    &sm.palettes[&(tileset.palette as usize)],
                    &sm.gfx_with_cre(tileset.graphic as usize),
                ),
                expected_image.as_rgb8().unwrap()
            );
        } else {
            assert_eq!(
                &tileset_to_image(
                    &sm.tile_tables[&(tileset.tile_table as usize)],
                    &sm.palettes[&(tileset.palette as usize)],
                    &sm.graphics[&(tileset.graphic as usize)],
                ),
                expected_image.as_rgb8().unwrap()
            );
        };
    }
}

/// Convert Rooms to image.
#[test]
#[ignore = "Requires ROM data to run, which is Copyrighted."]
fn convert_super_metroid_rooms_to_image() {
    let sm = super_metroid::load_unheadered_rom(
        fs::read("/home/rondao/dev/snes_data/test/Super Metroid (JU) [!].smc").unwrap(),
    )
    .unwrap();

    for (room_address, room) in sm.rooms.iter() {
        let expected_image = image::open(format!(
            "/home/rondao/dev/snes_data/test/{:x}.png",
            *room_address
        ))
        .unwrap();

        let room_image = sm.room_to_image(
            room,
            &sm.states[&room.state_conditions[0].state_address.into()],
        );
        assert_eq!(&room_image, expected_image.as_rgb8().unwrap());
    }
}
