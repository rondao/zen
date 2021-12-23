pub mod address;
pub mod door_list;
pub mod level_data;
pub mod room;
pub mod state;
pub mod tileset;

use std::{error::Error, fs};

use crate::{
    address::{LoRom, Pc},
    compress,
    graphics::{
        gfx::{self, Gfx, Tile8},
        palette, Palette,
    },
    ParseError,
};

use address::{CRE_GFX, CRE_TILESET, DOORS, GRAPHICS, PALETTES, ROOMS, TILESETS};
use door_list::Doors;
use level_data::Levels;
use room::Room;
use state::States;
use tileset::Tileset;

// "21f3e98df4780ee1c667b84e57d88675"
pub const UNHEADERED_MD5: [u8; 16] = [
    33, 243, 233, 141, 244, 120, 14, 225, 198, 103, 184, 78, 87, 216, 134, 117,
];

#[derive(Debug, Default, Clone)]
pub struct SuperMetroid {
    pub rom: Vec<u8>,
    pub palettes: Vec<Palette>,
    pub graphics: Vec<Gfx>,
    pub tilesets: Vec<Tileset>,
    pub cre_gfx: Gfx,
    pub cre_tileset: Tileset,
    pub levels: Levels,
    pub rooms: Vec<Room>,
    pub states: States,
    pub doors: Doors,
}

pub trait Offset {
    fn offset(&self, start: Pc) -> &[u8];
}

impl Offset for Vec<u8> {
    fn offset(&self, start: Pc) -> &[u8] {
        &self[start.address..]
    }
}

impl SuperMetroid {
    pub fn gfx_with_cre(&self, gfx: usize) -> Gfx {
        Gfx {
            tiles: [
                &self.graphics[gfx].tiles[..],
                &[Tile8 { colors: [0; 64] }; 64],
                &self.cre_gfx.tiles[..],
            ]
            .concat(),
        }
    }

    pub fn tileset_with_cre(&self, tileset: usize) -> Tileset {
        [&self.cre_tileset[..], &self.tilesets[tileset]].concat()
    }

    fn check_md5(&self) -> bool {
        md5::compute(&self.rom).0 == UNHEADERED_MD5
    }
}

pub fn load_unheadered_rom(filename: &str) -> Result<SuperMetroid, Box<dyn Error>> {
    let mut sm = SuperMetroid {
        rom: fs::read(filename)?,
        ..Default::default()
    };

    if !sm.check_md5() {
        return Err(Box::new(ParseError));
    }

    // Load all Palettes.
    for address in PALETTES {
        sm.palettes
            .push(palette::from_bytes(&compress::decompress_lz5(
                sm.rom.offset(LoRom { address: *address }.into()),
            )?)?);
    }

    // Load all Graphics.
    for address in GRAPHICS {
        sm.graphics.push(gfx::from_4bpp(&compress::decompress_lz5(
            sm.rom.offset(LoRom { address: *address }.into()),
        )?));
    }

    // Load all Tilesets.
    for address in TILESETS {
        sm.tilesets
            .push(tileset::from_bytes(&compress::decompress_lz5(
                sm.rom.offset(LoRom { address: *address }.into()),
            )?));
    }

    // Load CRE graphic.
    sm.cre_gfx = gfx::from_4bpp(&compress::decompress_lz5(
        sm.rom.offset(LoRom { address: CRE_GFX }.into()),
    )?);

    // Load CRE tileset.
    sm.cre_tileset = tileset::from_bytes(&compress::decompress_lz5(
        sm.rom.offset(
            LoRom {
                address: CRE_TILESET,
            }
            .into(),
        ),
    )?);

    // Load all Rooms.
    for address in ROOMS {
        sm.rooms.push(room::from_bytes(
            *address as u16,
            sm.rom.offset(LoRom { address: *address }.into()),
        ));
    }

    // Load all Doors.
    for door in DOORS {
        sm.doors.load_bytes(
            door.1 as u16,
            door.0,
            sm.rom.offset(
                LoRom {
                    address: 0x8F_0000 + door.1 as usize,
                }
                .into(),
            ),
        );
    }

    // Load all StateConditions, States and LevelData.
    for room in sm.rooms.iter() {
        for state_condition in room.state_conditions.iter() {
            sm.states.load_bytes(
                state_condition.state_address as usize,
                &sm.rom.offset(
                    LoRom {
                        address: 0x8F_0000 + state_condition.state_address as usize,
                    }
                    .into(),
                ),
            );

            let state = sm.states.get_state(state_condition.state_address as usize);
            sm.levels.load_from_bytes(
                state.level_data as usize,
                &compress::decompress_lz5(
                    &sm.rom.offset(
                        LoRom {
                            address: state.level_data as usize,
                        }
                        .into(),
                    ),
                )?,
                state.layer_2_x_scroll & state.layer_2_y_scroll & 1 == 0,
            );
        }
    }

    Ok(sm)
}
