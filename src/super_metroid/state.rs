use std::collections::HashMap;

//https://wiki.metroidconstruction.com/doku.php?id=super:technical_information:data_structures#state_header
#[derive(Debug, Default, Clone)]
pub struct State {
    pub level_data: u32, // Only three bytes are used (u24).
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

#[derive(Debug, Default, Clone)]
pub struct States {
    states: HashMap<usize, State>,
}

impl States {
    pub fn load_bytes(&mut self, state_address: usize, source: &[u8]) {
        if let None = self.states.get(&state_address) {
            #[rustfmt::skip]
            let state = State {
                level_data:           u32::from_le_bytes([source[0], source[1], source[2], 0]),
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
            };
            self.states.insert(state_address, state);
        }
    }

    pub fn get_state(&self, state_address: usize) -> &State {
        self.states.get(&state_address).unwrap()
    }
}