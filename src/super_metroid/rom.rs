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

use super::{
    address::{CRE_GFX, CRE_TILESET, DOORS, GRAPHICS, PALETTES, ROOMS, TILESETS},
    door_list::Doors,
    level_data::Levels,
    room::{self, Room},
    state::States,
    tileset::{self, Tileset},
};

// "21f3e98df4780ee1c667b84e57d88675"
pub const UNHEADERED_MD5: [u8; 16] = [
    33, 243, 233, 141, 244, 120, 14, 225, 198, 103, 184, 78, 87, 216, 134, 117,
];

#[derive(Debug, Default, Clone)]
pub struct Rom {
    rom: Vec<u8>,
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

impl Rom {
    pub fn offset(&self, start: Pc) -> &[u8] {
        &self.rom[start.address..]
    }

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

pub fn load_unheadered_rom(filename: &str) -> Result<Rom, Box<dyn Error>> {
    let mut rom = Rom {
        rom: fs::read(filename)?,
        ..Default::default()
    };

    if !rom.check_md5() {
        return Err(Box::new(ParseError));
    }

    // Load all Palettes.
    for address in PALETTES {
        rom.palettes
            .push(palette::from_bytes(&compress::decompress_lz5(
                rom.offset(LoRom { address: *address }.into()),
            )?)?);
    }

    // Load all Graphics.
    for address in GRAPHICS {
        rom.graphics.push(gfx::from_4bpp(&compress::decompress_lz5(
            rom.offset(LoRom { address: *address }.into()),
        )?));
    }

    // Load all Tilesets.
    for address in TILESETS {
        rom.tilesets
            .push(tileset::from_bytes(&compress::decompress_lz5(
                rom.offset(LoRom { address: *address }.into()),
            )?));
    }

    // Load CRE graphic.
    rom.cre_gfx = gfx::from_4bpp(&compress::decompress_lz5(
        rom.offset(LoRom { address: CRE_GFX }.into()),
    )?);

    // Load CRE tileset.
    rom.cre_tileset = tileset::from_bytes(&compress::decompress_lz5(
        rom.offset(
            LoRom {
                address: CRE_TILESET,
            }
            .into(),
        ),
    )?);

    // Load all Rooms.
    for address in ROOMS {
        rom.rooms.push(room::from_bytes(
            *address as u16,
            rom.offset(LoRom { address: *address }.into()),
        ));
    }

    // Load all Doors.
    for door in DOORS {
        let pc_door: Pc = LoRom {
            address: 0x8F_0000 + door.1 as usize,
        }
        .into();

        rom.doors
            .load_bytes(door.1 as u16, door.0, &rom.rom[pc_door.address..]);
    }

    // Load all StateConditions, States and LevelData.
    for room in rom.rooms.iter() {
        for state_condition in room.state_conditions.iter() {
            let pc_state: Pc = LoRom {
                address: 0x8F_0000 + state_condition.state_address as usize,
            }
            .into();

            rom.states.load_bytes(
                state_condition.state_address as usize,
                &rom.rom[pc_state.address..],
            );

            let state = rom.states.get_state(state_condition.state_address as usize);
            let pc_leveldata: Pc = LoRom {
                address: state.level_data as usize,
            }
            .into();

            rom.levels.load_from_bytes(
                state.level_data as usize,
                &compress::decompress_lz5(&rom.rom[pc_leveldata.address..])?,
                state.layer_2_x_scroll & state.layer_2_y_scroll & 1 == 0,
            );
        }
    }

    Ok(rom)
}
