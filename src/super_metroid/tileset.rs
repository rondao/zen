pub const TILESET_SIZE: usize = 9;
pub const NUMBER_OF_TILESETS: usize = 0x1D;

#[derive(Debug, Default, Clone, Copy)]
pub struct Tileset {
    pub palette: u32,
    pub graphic: u32,
    pub tile_table: u32,
    pub use_cre: bool,
}

#[rustfmt::skip]
pub fn from_bytes(source: &[u8]) -> Vec<Tileset> {
    source
        .chunks(TILESET_SIZE).enumerate()
        .map(|(index, data)| {
            Tileset {
                tile_table: u32::from_le_bytes([data[0], data[1], data[2], 0]),
                graphic:    u32::from_le_bytes([data[3], data[4], data[5], 0]),
                palette:    u32::from_le_bytes([data[6], data[7], data[8], 0]),
                use_cre:    !(0xF..0x14).contains(&index),
            }
        })
        .collect()
}
