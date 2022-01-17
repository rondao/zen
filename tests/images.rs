use zen::{
    image::tileset_to_image,
    super_metroid::{self, address::ROOMS},
};

/// Convert Palettes to image.
#[test]
fn convert_super_metroid_palettes_to_image() {
    let sm = super_metroid::load_unheadered_rom(
        "/home/rondao/dev/snes_data/test/Super Metroid (JU) [!].smc",
    )
    .unwrap();

    for (address, palette) in sm.palettes.iter() {
        let expected_image =
            image::open(format!("/home/rondao/dev/snes_data/test/{:x}.png", address)).unwrap();
        assert_eq!(&palette.to_image(), expected_image.as_rgb8().unwrap());
    }
}

/// Convert Gfx to image.
#[test]
fn convert_super_metroid_gfxs_to_image() {
    let sm = super_metroid::load_unheadered_rom(
        "/home/rondao/dev/snes_data/test/Super Metroid (JU) [!].smc",
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
fn convert_super_metroid_tilesets_to_image() {
    let sm = super_metroid::load_unheadered_rom(
        "/home/rondao/dev/snes_data/test/Super Metroid (JU) [!].smc",
    )
    .unwrap();

    for (i, tileset) in sm.tilesets.iter().enumerate() {
        let expected_image =
            image::open(format!("/home/rondao/dev/snes_data/test/tileset_{}.png", i)).unwrap();
        let image = if tileset.use_cre {
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
fn convert_super_metroid_rooms_to_image() {
    let sm = super_metroid::load_unheadered_rom(
        "/home/rondao/dev/snes_data/test/Super Metroid (JU) [!].smc",
    )
    .unwrap();

    for address in ROOMS {
        let expected_image = image::open(format!(
            "/home/rondao/dev/snes_data/test/{:x}.png",
            *address
        ))
        .unwrap();
        let room_image = sm.room_to_image(*address, 0).unwrap();

        assert_eq!(&room_image, expected_image.as_rgb8().unwrap());
    }
}
