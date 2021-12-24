use std::convert::TryInto;

#[derive(Debug, Default, Clone, Copy)]
pub struct Tileset {
    pub palette: usize,
    pub graphic: usize,
    pub tile_table: usize,
    pub use_cre: bool,
}

pub type TileTable = Vec<Tile>;

#[derive(Debug, Default, Clone, Copy)]
pub struct Tile {
    pub y_flip: bool,
    pub x_flip: bool,
    pub unknown: bool,
    pub sub_palette: u8,
    pub gfx_index: u16,
}

#[rustfmt::skip]
pub fn from_bytes(source: &[u8]) -> TileTable {
    source
        .chunks(2)
        .map(|tile_data| {
            let two_bytes = u16::from_le_bytes(tile_data.try_into().unwrap());
            Tile {
                y_flip:       (two_bytes & 0b1000_0000_0000_0000) != 0,
                x_flip:       (two_bytes & 0b0100_0000_0000_0000) != 0,
                unknown:      (two_bytes & 0b0010_0000_0000_0000) != 0,
                sub_palette: ((two_bytes & 0b0001_1100_0000_0000) >> 10) as u8,
                gfx_index:    (two_bytes & 0b0000_0011_1111_1111) as u16,
            }
        })
        .collect()
}

pub const TILESETS: [Tileset; 0x1D] = [
    // Graphic set 0: Upper Crateria
    Tileset {
        palette: 0,
        graphic: 0,
        tile_table: 0,
        use_cre: true,
    },
    // Graphic set 1: Red Crateria
    Tileset {
        palette: 1,
        graphic: 0,
        tile_table: 0,
        use_cre: true,
    },
    // Graphic set 2: Lower Crateria
    Tileset {
        palette: 2,
        graphic: 1,
        tile_table: 1,
        use_cre: true,
    },
    // Graphic set 3: Old Tourian
    Tileset {
        palette: 3,
        graphic: 1,
        tile_table: 1,
        use_cre: true,
    },
    // Graphic set 4: Wrecked Ship - power on
    Tileset {
        palette: 4,
        graphic: 2,
        tile_table: 2,
        use_cre: true,
    },
    // Graphic set 5: Wrecked Ship - power off
    Tileset {
        palette: 5,
        graphic: 2,
        tile_table: 2,
        use_cre: true,
    },
    // Graphic set 6: Green/blue Brinstar
    Tileset {
        palette: 6,
        graphic: 3,
        tile_table: 3,
        use_cre: true,
    },
    // Graphic set 7: Red Brinstar / Kraid's lair
    Tileset {
        palette: 7,
        graphic: 4,
        tile_table: 4,
        use_cre: true,
    },
    // Graphic set 8: Pre Tourian entrance corridor
    Tileset {
        palette: 8,
        graphic: 4,
        tile_table: 4,
        use_cre: true,
    },
    // Graphic set 9: Heated Norfair
    Tileset {
        palette: 10,
        graphic: 6,
        tile_table: 6,
        use_cre: true,
    },
    // Graphic set Ah: Unheated Norfair
    Tileset {
        palette: 11,
        graphic: 6,
        tile_table: 6,
        use_cre: true,
    },
    // Graphic set Bh: Sandless Maridia
    Tileset {
        palette: 13,
        graphic: 8,
        tile_table: 8,
        use_cre: true,
    },
    // Graphic set Ch: Sandy Maridia
    Tileset {
        palette: 14,
        graphic: 9,
        tile_table: 9,
        use_cre: true,
    },
    // Graphic set Dh: Tourian
    Tileset {
        palette: 16,
        graphic: 11,
        tile_table: 11,
        use_cre: true,
    },
    // Graphic set Eh: Mother Brain's room
    Tileset {
        palette: 17,
        graphic: 11,
        tile_table: 11,
        use_cre: true,
    },
    // Graphic set Fh: Blue Ceres
    Tileset {
        palette: 23,
        graphic: 13,
        tile_table: 13,
        use_cre: false,
    },
    // Graphic set 10h: White Ceres
    Tileset {
        palette: 24,
        graphic: 13,
        tile_table: 13,
        use_cre: false,
    },
    // Graphic set 11h: Blue Ceres Elevator
    Tileset {
        palette: 23,
        graphic: 14,
        tile_table: 13,
        use_cre: false,
    },
    // Graphic set 12h: White Ceres Elevator
    Tileset {
        palette: 24,
        graphic: 14,
        tile_table: 13,
        use_cre: false,
    },
    // Graphic set 13h: Blue Ceres Ridley's room
    Tileset {
        palette: 23,
        graphic: 15,
        tile_table: 13,
        use_cre: false,
    },
    // Graphic set 14h: White Ceres Ridley's room
    Tileset {
        palette: 24,
        graphic: 15,
        tile_table: 13,
        use_cre: false,
    },
    // Graphic set 15h: Map room / Tourian entrance
    Tileset {
        palette: 18,
        graphic: 12,
        tile_table: 12,
        use_cre: true,
    },
    // Graphic set 16h: Wrecked Ship map room - power off
    Tileset {
        palette: 19,
        graphic: 12,
        tile_table: 12,
        use_cre: true,
    },
    // Graphic set 17h: Blue refill room
    Tileset {
        palette: 20,
        graphic: 12,
        tile_table: 12,
        use_cre: true,
    },
    // Graphic set 18h: Yellow refill room
    Tileset {
        palette: 21,
        graphic: 12,
        tile_table: 12,
        use_cre: true,
    },
    // Graphic set 19h: Save room
    Tileset {
        palette: 22,
        graphic: 12,
        tile_table: 12,
        use_cre: true,
    },
    // Graphic set 1A: Kraid's room
    Tileset {
        palette: 9,
        graphic: 5,
        tile_table: 5,
        use_cre: true,
    },
    // Graphic set 1Bh: Crocomire's room
    Tileset {
        palette: 12,
        graphic: 7,
        tile_table: 7,
        use_cre: true,
    },
    // Graphic set 1Ch: Draygon's room
    Tileset {
        palette: 15,
        graphic: 10,
        tile_table: 10,
        use_cre: true,
    },
];
