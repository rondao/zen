#![feature(test)]

extern crate test;
use test::Bencher;
use zen::super_metroid::{self, tileset::tileset_to_colors};

#[bench]
fn bench_palette_to_colors(b: &mut Bencher) {
    let sm = super_metroid::load_unheadered_rom(
        std::fs::read("/home/rondao/dev/snes_data/test/Super Metroid (JU) [!].smc").unwrap(),
    )
    .unwrap();

    b.iter(|| -> Vec<_> {
        sm.palettes
            .values()
            .map(|palette| palette.to_colors())
            .collect()
    });
}

#[bench]
fn bench_gfx_to_colors(b: &mut Bencher) {
    let sm = super_metroid::load_unheadered_rom(
        std::fs::read("/home/rondao/dev/snes_data/test/Super Metroid (JU) [!].smc").unwrap(),
    )
    .unwrap();

    b.iter(|| -> Vec<_> {
        sm.graphics
            .values()
            .map(|gfx| gfx.to_indexed_colors())
            .collect()
    });
}

#[bench]
fn bench_tileset_to_colors(b: &mut Bencher) {
    let sm = super_metroid::load_unheadered_rom(
        std::fs::read("/home/rondao/dev/snes_data/test/Super Metroid (JU) [!].smc").unwrap(),
    )
    .unwrap();

    b.iter(|| -> Vec<_> {
        sm.tilesets
            .iter()
            .map(|tileset| {
                tileset_to_colors(
                    &sm.tile_table_with_cre(tileset.tile_table as usize),
                    &sm.palettes[&(tileset.palette as usize)],
                    &sm.gfx_with_cre(tileset.graphic as usize),
                )
            })
            .collect()
    });
}

#[bench]
fn bench_room_to_colors(b: &mut Bencher) {
    let sm = super_metroid::load_unheadered_rom(
        std::fs::read("/home/rondao/dev/snes_data/test/Super Metroid (JU) [!].smc").unwrap(),
    )
    .unwrap();

    b.iter(|| -> Vec<_> {
        sm.rooms
            .values()
            .map(|room| {
                let (level_data, _, palette, graphics, tile_table) =
                    sm.get_state_data(&sm.states[&room.state_conditions[0].state_address.into()]);
                level_data.to_colors(room.size(), &tile_table, palette, &graphics)
            })
            .collect()
    });
}
