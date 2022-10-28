pub mod address;
pub mod door;
pub mod door_list;
pub mod level_data;
pub mod room;
pub mod save_station;
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
    compress::{lz5_compress, lz5_decompress},
    graphics::{
        gfx::{self, Gfx, TileGfx},
        palette, Palette,
    },
    ParseError,
};

use address::{CRE_GFX, CRE_TILESET, DOORS_LIST, ROOMS, TILESETS};
use door_list::DoorList;
use level_data::LevelData;
use room::Room;
use state::State;
use tile_table::TileTable;

use self::{
    address::DOORS,
    door::{Door, DOOR_BYTE_SIZE},
    save_station::SaveStation,
    tileset::Tileset,
};

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
    pub doors: HashMap<usize, Door>,
    pub door_lists: HashMap<usize, DoorList>,
    pub save_stations: Vec<Vec<SaveStation>>,
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

    pub fn get_state_data(
        &self,
        state: &State,
    ) -> (&LevelData, &Tileset, &Palette, Gfx, TileTable) {
        let level_data = &self.levels[&(state.level_address as usize)];
        let tileset = &self.tilesets[state.tileset as usize];

        let palette = &self.palettes[&(tileset.palette as usize)];

        let graphics = if tileset.use_cre {
            self.gfx_with_cre(tileset.graphic as usize)
        } else {
            self.graphics[&(tileset.graphic as usize)].clone()
        };

        let tile_table = if tileset.use_cre {
            self.tile_table_with_cre(tileset.tile_table as usize)
        } else {
            self.tile_tables[&(tileset.tile_table as usize)].clone()
        };

        (level_data, tileset, palette, graphics, tile_table)
    }

    pub fn save_to_rom(&mut self) {
        self.save_palettes_to_rom();
        self.save_level_data_to_rom();

        // Write tilesets to ROM.
        let tileset_address: Pc = LoRom { address: TILESETS }.into();
        let tilesets_as_bytes = self.tilesets.iter().fold(Vec::new(), |mut accum, tileset| {
            accum.extend(tileset.to_bytes());
            accum
        });
        self.rom.splice(
            tileset_address.address..tileset_address.address + tilesets_as_bytes.len(),
            tilesets_as_bytes,
        );
    }

    pub fn save_to_file(&mut self, filename: &str) -> std::io::Result<()> {
        fs::write(filename, &self.rom)
    }

    pub fn save_palettes_to_rom(&mut self) -> HashMap<usize, usize> {
        let mut remapped_addresses: HashMap<usize, usize> = HashMap::new();
        let mut pc_to_write: Pc = LoRom { address: 0xC2AD7C }.into();

        // Compress every palette and save to ROM.
        for (pal_address, palette) in self.palettes.iter() {
            let pal_compressed_bytes = lz5_compress(&palette.to_bytes());
            let number_of_bytes = pal_compressed_bytes.len();

            self.rom.splice(
                pc_to_write.address..pc_to_write.address + number_of_bytes,
                pal_compressed_bytes,
            );

            remapped_addresses.insert(*pal_address, LoRom::from(pc_to_write).address);
            pc_to_write.address += number_of_bytes;
        }

        // Update palette list addresses.
        self.palettes =
            self.palettes
                .iter()
                .fold(HashMap::new(), |mut accum, (address, palette)| {
                    accum.insert(remapped_addresses[address], *palette);
                    accum
                });

        // Tileset addresses references needs to be changed accordingly.
        for tileset in self.tilesets.iter_mut() {
            tileset.palette = remapped_addresses[&(tileset.palette as usize)] as u32;
        }

        remapped_addresses
    }

    pub fn save_level_data_to_rom(&mut self) -> HashMap<usize, usize> {
        let mut remapped_addresses: HashMap<usize, usize> = HashMap::new();
        let mut pc_to_write: Pc = LoRom { address: 0xC2C2BB }.into();

        // Compress every level data and save to ROM.
        for (level_address, level) in self.levels.iter() {
            let level_compressed_bytes = lz5_compress(&level.to_bytes());
            let number_of_bytes = level_compressed_bytes.len();

            self.rom.splice(
                pc_to_write.address..pc_to_write.address + number_of_bytes,
                level_compressed_bytes,
            );

            remapped_addresses.insert(*level_address, LoRom::from(pc_to_write).address);
            pc_to_write.address += number_of_bytes;
        }

        // Update levels list addresses.
        self.levels = self
            .levels
            .iter()
            .fold(HashMap::new(), |mut accum, (address, level)| {
                accum.insert(remapped_addresses[address], level.clone());
                accum
            });

        // State addresses references to Levels needs to be changed accordingly.
        for state in self.states.values_mut() {
            state.level_address = remapped_addresses[&(state.level_address as usize)] as u32;
        }

        // Save all Rooms in-place. TODO: They should be saved in any place.
        for (room_address, room) in &self.rooms {
            let room_data = room.to_bytes();
            let pc_to_write: Pc = LoRom {
                address: *room_address,
            }
            .into();

            self.rom.splice(
                pc_to_write.address..pc_to_write.address + room_data.len(),
                room_data,
            );
        }

        remapped_addresses
    }

    fn check_md5(&self) -> bool {
        md5::compute(&self.rom).0 == UNHEADERED_MD5
    }
}

pub fn load_unheadered_rom(data: Vec<u8>) -> Result<SuperMetroid, Box<dyn Error>> {
    let mut sm = SuperMetroid {
        rom: data,
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
            .or_insert(palette::from_bytes(&lz5_decompress(
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
            .or_insert(gfx::from_4bpp(&lz5_decompress(
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
            .or_insert(tile_table::from_bytes(&lz5_decompress(
                sm.rom.offset(
                    LoRom {
                        address: tileset.tile_table as usize,
                    }
                    .into(),
                ),
            )?));
    }

    // Load CRE graphic.
    sm.cre_gfx = gfx::from_4bpp(&lz5_decompress(
        sm.rom.offset(LoRom { address: CRE_GFX }.into()),
    )?);

    // Load CRE tileset.
    sm.cre_tileset = tile_table::from_bytes(&lz5_decompress(
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

    // Load all Door Lists.
    for door_list in DOORS_LIST {
        sm.door_lists
            .entry(door_list.1)
            .or_insert(door_list::load_bytes(
                door_list.0,
                sm.rom.offset(
                    LoRom {
                        address: 0x8F_0000 + door_list.1 as usize,
                    }
                    .into(),
                ),
            ));
    }

    // Load all Doors.
    let mut address: usize = DOORS.1;
    for door in door::load_bytes(DOORS.0, sm.rom.offset(LoRom { address }.into())) {
        sm.doors.insert(address, door);
        address += DOOR_BYTE_SIZE;
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
                    if let Ok(decompressed_data) = &lz5_decompress(
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

    // Load all Save Stations.
    sm.save_stations = save_station::load_all_from_list(
        sm.rom.offset(
            LoRom {
                address: address::SAVE_STATIONS,
            }
            .into(),
        ),
        sm.rom.offset(
            LoRom {
                address: address::SAVE_STATIONS_LIST,
            }
            .into(),
        ),
        address::NUMBER_OF_AREAS,
    );

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

#[cfg(test)]
mod tests {
    use super::{tile_table::Tile, *};

    /// Load Super Metroid data from rom.
    #[test]
    fn load_super_metroid_data_from_rom() {
        assert!(load_unheadered_rom(
            fs::read("/home/rondao/dev/snes_data/test/Super Metroid (JU) [!].smc").unwrap()
        )
        .is_ok());
    }

    /// Fail to load Super Metroid data from incorrect rom data.
    #[test]
    fn load_super_metroid_data_from_incorrect_rom() {
        assert!(load_unheadered_rom(
            fs::read("/home/rondao/dev/snes_data/test/Incorrect Super Metroid.smc").unwrap()
        )
        .is_err());
    }

    /// Get a Gfx with CRE.
    #[test]
    fn get_gfx_with_cre() {
        let mut graphics = HashMap::new();
        graphics.insert(
            0xFF,
            Gfx {
                tiles: vec![TileGfx {
                    colors: [
                        0b1110, 0b1101, 0b1100, 0b1011, 0b1010, 0b1001, 0b1000, 0b0111, 0b1101,
                        0b1100, 0b1011, 0b1010, 0b1001, 0b1000, 0b0111, 0b0110, 0b1100, 0b1011,
                        0b1010, 0b1001, 0b1000, 0b0111, 0b0110, 0b0101, 0b1011, 0b1010, 0b1001,
                        0b1000, 0b0111, 0b0110, 0b0101, 0b0100, 0b1010, 0b1001, 0b1000, 0b0111,
                        0b0110, 0b0101, 0b0100, 0b0011, 0b1001, 0b1000, 0b0111, 0b0110, 0b0101,
                        0b0100, 0b0011, 0b0010, 0b1000, 0b0111, 0b0110, 0b0101, 0b0100, 0b0011,
                        0b0010, 0b0001, 0b0111, 0b0110, 0b0101, 0b0100, 0b0011, 0b0010, 0b0001,
                        0b0000,
                    ],
                }],
            },
        );

        let sm = SuperMetroid {
            cre_gfx: Gfx {
                tiles: vec![TileGfx {
                    colors: [
                        0b0001, 0b0010, 0b0011, 0b0100, 0b0101, 0b0110, 0b0111, 0b1000, 0b0010,
                        0b0011, 0b0100, 0b0101, 0b0110, 0b0111, 0b1000, 0b1001, 0b0011, 0b0100,
                        0b0101, 0b0110, 0b0111, 0b1000, 0b1001, 0b1010, 0b0100, 0b0101, 0b0110,
                        0b0111, 0b1000, 0b1001, 0b1010, 0b1011, 0b0101, 0b0110, 0b0111, 0b1000,
                        0b1001, 0b1010, 0b1011, 0b1100, 0b0110, 0b0111, 0b1000, 0b1001, 0b1010,
                        0b1011, 0b1100, 0b1101, 0b0111, 0b1000, 0b1001, 0b1010, 0b1011, 0b1100,
                        0b1101, 0b1110, 0b1000, 0b1001, 0b1010, 0b1011, 0b1100, 0b1101, 0b1110,
                        0b1111,
                    ],
                }],
            },
            graphics,
            ..Default::default()
        };

        let expected_gfx_with_cre = Gfx {
            tiles: [
                vec![TileGfx {
                    colors: [
                        0b1110, 0b1101, 0b1100, 0b1011, 0b1010, 0b1001, 0b1000, 0b0111, 0b1101,
                        0b1100, 0b1011, 0b1010, 0b1001, 0b1000, 0b0111, 0b0110, 0b1100, 0b1011,
                        0b1010, 0b1001, 0b1000, 0b0111, 0b0110, 0b0101, 0b1011, 0b1010, 0b1001,
                        0b1000, 0b0111, 0b0110, 0b0101, 0b0100, 0b1010, 0b1001, 0b1000, 0b0111,
                        0b0110, 0b0101, 0b0100, 0b0011, 0b1001, 0b1000, 0b0111, 0b0110, 0b0101,
                        0b0100, 0b0011, 0b0010, 0b1000, 0b0111, 0b0110, 0b0101, 0b0100, 0b0011,
                        0b0010, 0b0001, 0b0111, 0b0110, 0b0101, 0b0100, 0b0011, 0b0010, 0b0001,
                        0b0000,
                    ],
                }],
                vec![TileGfx { colors: [0; 64] }; 64],
                vec![TileGfx {
                    colors: [
                        0b0001, 0b0010, 0b0011, 0b0100, 0b0101, 0b0110, 0b0111, 0b1000, 0b0010,
                        0b0011, 0b0100, 0b0101, 0b0110, 0b0111, 0b1000, 0b1001, 0b0011, 0b0100,
                        0b0101, 0b0110, 0b0111, 0b1000, 0b1001, 0b1010, 0b0100, 0b0101, 0b0110,
                        0b0111, 0b1000, 0b1001, 0b1010, 0b1011, 0b0101, 0b0110, 0b0111, 0b1000,
                        0b1001, 0b1010, 0b1011, 0b1100, 0b0110, 0b0111, 0b1000, 0b1001, 0b1010,
                        0b1011, 0b1100, 0b1101, 0b0111, 0b1000, 0b1001, 0b1010, 0b1011, 0b1100,
                        0b1101, 0b1110, 0b1000, 0b1001, 0b1010, 0b1011, 0b1100, 0b1101, 0b1110,
                        0b1111,
                    ],
                }],
            ]
            .concat(),
        };

        assert_eq!(sm.gfx_with_cre(0xFF), expected_gfx_with_cre);
    }

    /// Get a Tiletable with CRE.
    #[test]
    fn get_tile_table_with_cre() {
        let mut tile_tables = HashMap::new();
        tile_tables.insert(
            0xFF,
            vec![
                Tile {
                    y_flip: false,
                    x_flip: false,
                    unknown: false,
                    sub_palette: 0,
                    gfx_index: 0,
                },
                Tile {
                    y_flip: false,
                    x_flip: true,
                    unknown: false,
                    sub_palette: 0b101,
                    gfx_index: 0b01_1010_0101,
                },
            ],
        );

        let sm = SuperMetroid {
            cre_tileset: vec![
                Tile {
                    y_flip: true,
                    x_flip: false,
                    unknown: true,
                    sub_palette: 0b010,
                    gfx_index: 0b10_0101_1010,
                },
                Tile {
                    y_flip: true,
                    x_flip: true,
                    unknown: true,
                    sub_palette: 0b111,
                    gfx_index: 0b11_1111_1111,
                },
            ],
            tile_tables,
            ..Default::default()
        };

        let expected_tile_table_with_cre = vec![
            Tile {
                y_flip: true,
                x_flip: false,
                unknown: true,
                sub_palette: 0b010,
                gfx_index: 0b10_0101_1010,
            },
            Tile {
                y_flip: true,
                x_flip: true,
                unknown: true,
                sub_palette: 0b111,
                gfx_index: 0b11_1111_1111,
            },
            Tile {
                y_flip: false,
                x_flip: false,
                unknown: false,
                sub_palette: 0,
                gfx_index: 0,
            },
            Tile {
                y_flip: false,
                x_flip: true,
                unknown: false,
                sub_palette: 0b101,
                gfx_index: 0b01_1010_0101,
            },
        ];

        assert_eq!(sm.tile_table_with_cre(0xFF), expected_tile_table_with_cre);
    }
}
