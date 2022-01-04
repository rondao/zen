pub mod address;
pub mod door_list;
pub mod level_data;
pub mod room;
pub mod state;
pub mod tile_table;
pub mod tileset;

use std::{
    collections::{hash_map::Entry, HashMap},
    error::Error,
    fs,
};

use crate::{
    address::{LoRom, Pc},
    compress,
    graphics::{
        gfx::{self, Gfx, TileGfx},
        palette, Palette,
    },
    ParseError,
};

use address::{CRE_GFX, CRE_TILESET, DOORS, ROOMS, TILESETS};
use door_list::DoorList;
use level_data::LevelData;
use room::Room;
use state::State;
use tile_table::TileTable;

use self::tileset::Tileset;

// "21f3e98df4780ee1c667b84e57d88675"
pub const UNHEADERED_MD5: [u8; 16] = [
    33, 243, 233, 141, 244, 120, 14, 225, 198, 103, 184, 78, 87, 216, 134, 117,
];

#[derive(Debug, Default, Clone)]
pub struct SuperMetroid {
    pub rom: Vec<u8>,
    pub cre_gfx: Gfx,
    pub cre_tileset: TileTable,
    pub tilesets: Vec<Tileset>,
    pub palettes: HashMap<usize, Palette>,
    pub graphics: HashMap<usize, Gfx>,
    pub tile_tables: HashMap<usize, TileTable>,
    pub levels: HashMap<usize, LevelData>,
    pub rooms: HashMap<usize, Room>,
    pub states: HashMap<usize, State>,
    pub doors: HashMap<usize, DoorList>,
}

impl SuperMetroid {
    pub fn gfx_with_cre(&self, gfx: usize) -> Gfx {
        Gfx {
            tiles: [
                &self.graphics[&gfx].tiles[..],
                &[TileGfx { colors: [0; 64] }; 64],
                &self.cre_gfx.tiles[..],
            ]
            .concat(),
        }
    }

    pub fn tile_table_with_cre(&self, tileset: usize) -> TileTable {
        [&self.cre_tileset[..], &self.tile_tables[&tileset]].concat()
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

    // Load all Tilesets.
    sm.tilesets = tileset::from_bytes(
        &sm.rom.offset(LoRom { address: TILESETS }.into())
            [..tileset::TILESET_DATA_SIZE * tileset::NUMBER_OF_TILESETS],
    );
    // Load all Tilesets.
    for tileset in sm.tilesets.iter() {
        // Load it's Palette.
        sm.palettes
            .entry(tileset.palette as usize)
            .or_insert(palette::from_bytes(&compress::decompress_lz5(
                sm.rom.offset(
                    LoRom {
                        address: tileset.palette as usize,
                    }
                    .into(),
                ),
            )?)?);

        // Load it's Graphics.
        sm.graphics
            .entry(tileset.graphic as usize)
            .or_insert(gfx::from_4bpp(&compress::decompress_lz5(
                sm.rom.offset(
                    LoRom {
                        address: tileset.graphic as usize,
                    }
                    .into(),
                ),
            )?));

        // Load all Tile Tables.
        sm.tile_tables
            .entry(tileset.tile_table as usize)
            .or_insert(tile_table::from_bytes(&compress::decompress_lz5(
                sm.rom.offset(
                    LoRom {
                        address: tileset.tile_table as usize,
                    }
                    .into(),
                ),
            )?));
    }

    // Load CRE graphic.
    sm.cre_gfx = gfx::from_4bpp(&compress::decompress_lz5(
        sm.rom.offset(LoRom { address: CRE_GFX }.into()),
    )?);

    // Load CRE tileset.
    sm.cre_tileset = tile_table::from_bytes(&compress::decompress_lz5(
        sm.rom.offset(
            LoRom {
                address: CRE_TILESET,
            }
            .into(),
        ),
    )?);

    // Load all Rooms.
    for address in ROOMS {
        sm.rooms.entry(*address).or_insert(room::from_bytes(
            *address as u16,
            sm.rom.offset(LoRom { address: *address }.into()),
        ));
    }

    // Load all Doors.
    for door in DOORS {
        sm.doors.entry(door.1).or_insert(door_list::load_bytes(
            door.0,
            sm.rom.offset(
                LoRom {
                    address: 0x8F_0000 + door.1 as usize,
                }
                .into(),
            ),
        ));
    }

    // Load all StateConditions, States and LevelData.
    for room in sm.rooms.values() {
        for state_condition in room.state_conditions.iter() {
            sm.states
                .entry(state_condition.state_address as usize)
                .or_insert(state::load_bytes(
                    &sm.rom.offset(
                        LoRom {
                            address: 0x8F_0000 + state_condition.state_address as usize,
                        }
                        .into(),
                    ),
                ));

            if let Some(state) = sm.states.get(&(state_condition.state_address as usize)) {
                if let Entry::Vacant(entry) = sm.levels.entry(state.level_address as usize) {
                    if let Ok(decompressed_data) = &compress::decompress_lz5(
                        &sm.rom.offset(
                            LoRom {
                                address: state.level_address as usize,
                            }
                            .into(),
                        ),
                    ) {
                        if let Ok(level) = level_data::load_from_bytes(
                            &decompressed_data,
                            state.layer_2_x_scroll & state.layer_2_y_scroll & 1 == 0,
                        ) {
                            entry.insert(level);
                        } else {
                            println!("Could not load Level Data at 0x{:x}", state.level_address);
                        }
                    } else {
                        println!(
                            "Could not decompress Level Data at 0x{:x}",
                            state.level_address
                        );
                    }
                }
            }
        }
    }

    Ok(sm)
}

pub trait Offset {
    fn offset(&self, start: Pc) -> &[u8];
}

impl Offset for Vec<u8> {
    fn offset(&self, start: Pc) -> &[u8] {
        &self[start.address..]
    }
}
