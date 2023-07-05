#![feature(test)]

extern crate test;
use test::Bencher;
use zen::super_metroid::{
    self,
    tileset::{tileset_to_colors, tileset_to_indexed_colors},
};

#[bench]
#[ignore = "Requires ROM data to run, which is Copyrighted."]
fn bench_palette_to_colors(b: &mut Bencher) {
    let sm = super_metroid::load_unheadered_rom(
        std::fs::read("/home/rondao/dev/snes_data/test/Super Metroid (JU) [!].smc").unwrap(),
    )
    .unwrap();

    let palette = &sm.palettes[&0xC2AD7C];
    b.iter(|| -> Vec<_> { palette.to_colors() });
}

#[bench]
#[ignore = "Requires ROM data to run, which is Copyrighted."]
fn bench_gfx_to_colors(b: &mut Bencher) {
    let sm = super_metroid::load_unheadered_rom(
        std::fs::read("/home/rondao/dev/snes_data/test/Super Metroid (JU) [!].smc").unwrap(),
    )
    .unwrap();

    let gfx = &sm.graphics[&0xBAC629];
    b.iter(|| -> Vec<_> { gfx.to_indexed_colors() });
}

#[bench]
#[ignore = "Requires ROM data to run, which is Copyrighted."]
fn bench_tileset_to_colors(b: &mut Bencher) {
    let sm = super_metroid::load_unheadered_rom(
        std::fs::read("/home/rondao/dev/snes_data/test/Super Metroid (JU) [!].smc").unwrap(),
    )
    .unwrap();

    let tileset = &sm.tilesets[0];
    b.iter(|| -> Vec<_> {
        tileset_to_colors(
            &sm.tile_table_with_cre(tileset.tile_table as usize),
            &sm.palettes[&(tileset.palette as usize)],
            &sm.gfx_with_cre(tileset.graphic as usize),
        )
    });
}

#[bench]
#[ignore = "Requires ROM data to run, which is Copyrighted."]
fn bench_tileset_to_indexed_colors(b: &mut Bencher) {
    let sm = super_metroid::load_unheadered_rom(
        std::fs::read("/home/rondao/dev/snes_data/test/Super Metroid (JU) [!].smc").unwrap(),
    )
    .unwrap();

    let tileset = &sm.tilesets[0];
    b.iter(|| -> Vec<_> {
        tileset_to_indexed_colors(
            &sm.tile_table_with_cre(tileset.tile_table as usize),
            &sm.gfx_with_cre(tileset.graphic as usize),
        )
    });
}

#[bench]
#[ignore = "Requires ROM data to run, which is Copyrighted."]
fn bench_room_to_colors(b: &mut Bencher) {
    let sm = super_metroid::load_unheadered_rom(
        std::fs::read("/home/rondao/dev/snes_data/test/Super Metroid (JU) [!].smc").unwrap(),
    )
    .unwrap();

    let room = &sm.rooms[&0x8F91F8];
    b.iter(|| -> Vec<_> {
        let (level_data, _, palette, graphics, tile_table) =
            sm.get_state_data(&sm.states[&room.state_conditions[0].state_address.into()]);
        level_data
            .to_colors(room.size(), &tile_table, palette, &graphics)
            .collect()
    });
}
