use std::convert::TryInto;

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
