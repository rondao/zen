//https://wiki.metroidconstruction.com/doku.php?id=super:technical_information:data_structures#state_header
#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub struct State {
    pub level_address: u32, // Only three bytes are used (u24).
    pub tileset: u8,
    pub music_data_index: u8,
    pub music_track: u8,
    pub fx: u16,
    pub enemy_population: u16,
    pub enemy_set: u16,
    pub layer_2_x_scroll: u8,
    pub layer_2_y_scroll: u8,
    pub scroll: u16,
    pub special_x_ray_blocks: u16,
    pub main_asm: u16,
    pub plm_population: u16,
    pub library_background: u16,
    pub setup_asm: u16,
}

#[rustfmt::skip]
pub fn load_bytes(source: &[u8]) -> State {
    State {
        level_address:        u32::from_le_bytes([source[0], source[1], source[2], 0]),
        tileset:              source[3],
        music_data_index:     source[4],
        music_track:          source[5],
        fx:                   u16::from_le_bytes([source[6], source[7]]),
        enemy_population:     u16::from_le_bytes([source[8], source[9]]),
        enemy_set:            u16::from_le_bytes([source[10], source[11]]),
        layer_2_x_scroll:     source[12],
        layer_2_y_scroll:     source[13],
        scroll:               u16::from_le_bytes([source[14], source[15]]),
        special_x_ray_blocks: u16::from_le_bytes([source[16], source[17]]),
        main_asm:             u16::from_le_bytes([source[18], source[19]]),
        plm_population:       u16::from_le_bytes([source[20], source[21]]),
        library_background:   u16::from_le_bytes([source[22], source[23]]),
        setup_asm:            u16::from_le_bytes([source[24], source[25]]),
    }
}

mod tests {
    use super::*;

    /// Load a State from bytes.
    #[test]
    fn load_state_from_bytes() {
        #[rustfmt::skip]
        let data = [
            0x56, 0x34, 0x12, // level_address
            0xA2,             // tileset
            0x9D,             // music_data_index
            0x38,             // music_track
            0x34, 0x12,       // fx
            0x78, 0x56,       // enemy_population
            0xBC, 0x9A,       // enemy_set
            0x00,             // layer_2_x_scroll
            0xFF,             // layer_2_y_scroll
            0x23, 0x01,       // scroll
            0x67, 0x45,       // special_x_ray_blocks
            0xAB, 0x89,       // main_asm
            0xEF, 0xCD,       // plm_population
            0x77, 0x66,       // library_background
            0xBB, 0xAA,       // setup_asm
        ];

        let expected_state = State {
            level_address: 0x123456,
            tileset: 0xA2,
            music_data_index: 0x9D,
            music_track: 0x38,
            fx: 0x1234,
            enemy_population: 0x5678,
            enemy_set: 0x9ABC,
            layer_2_x_scroll: 0x00,
            layer_2_y_scroll: 0xFF,
            scroll: 0x0123,
            special_x_ray_blocks: 0x4567,
            main_asm: 0x89AB,
            plm_population: 0xCDEF,
            library_background: 0x6677,
            setup_asm: 0xAABB,
        };

        assert_eq!(load_bytes(&data), expected_state);
    }
}
