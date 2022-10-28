use crate::graphics::gfx::TILE_SIZE;

use super::level_data::BLOCKS_PER_SCREEN;

/// Room format reference: https://wiki.metroidconstruction.com/doku.php?id=super:technical_information:data_structures#room_header
#[derive(Debug, Default, Clone, PartialEq)]
pub struct Room {
    pub index: u8,
    pub area: u8,
    pub map_position: (u8, u8),
    pub width: u8,
    pub height: u8,
    pub up_scroller: u8,
    pub down_scroller: u8,
    pub cre_bitset: u8,
    pub doors: u16,
    pub state_conditions: Vec<StateCondition>,
}

impl Room {
    pub fn size(&self) -> (usize, usize) {
        (self.width as usize, self.height as usize)
    }

    pub fn size_in_blocks(&self) -> [usize; 2] {
        [
            (BLOCKS_PER_SCREEN * self.size().0),
            (BLOCKS_PER_SCREEN * self.size().1),
        ]
    }

    pub fn size_in_pixels(&self) -> [usize; 2] {
        [
            (BLOCKS_PER_SCREEN * TILE_SIZE * 2 * self.size().0),
            (BLOCKS_PER_SCREEN * TILE_SIZE * 2 * self.size().1),
        ]
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut output = Vec::new();

        output.extend([
            self.index,
            self.area,
            self.map_position.0,
            self.map_position.1,
            self.width,
            self.height,
            self.up_scroller,
            self.down_scroller,
            self.cre_bitset,
        ]);
        output.extend(self.doors.to_le_bytes());

        output.extend(self.state_conditions.iter().rev().fold(
            Vec::new(),
            |mut acc, state_condition| {
                acc.extend(state_condition.to_bytes());
                acc
            },
        ));

        output
    }
}

pub fn from_bytes(room_address: u16, source: &[u8]) -> Room {
    #[rustfmt::skip]
    let room = Room {
        index:         source[0],
        area:          source[1],
        map_position: (source[2],
                       source[3]),
        width:         source[4],
        height:        source[5],
        up_scroller:   source[6],
        down_scroller: source[7],
        cre_bitset:    source[8],
        doors:         u16::from_le_bytes([source[9], source[10]]),
        state_conditions: state_conditions_from_bytes(room_address + 11, &source[11..]),
    };
    room
}

/// Room format reference: https://wiki.metroidconstruction.com/doku.php?id=super:technical_information:data_structures#room_header
#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub struct StateCondition {
    pub condition: u16,
    pub parameter: Option<u16>,
    pub state_address: u16,
}

impl StateCondition {
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut output = Vec::new();

        output.extend(self.condition.to_le_bytes());
        if let Some(parameter) = self.parameter {
            if self.condition == 0xE5EB {
                output.extend(parameter.to_le_bytes());
            } else if self.condition == 0xE612 || self.condition == 0xE629 {
                output.push(parameter as u8);
            }
        }
        if self.condition != 0xE5E6 {
            output.extend(self.state_address.to_le_bytes());
        }

        output
    }
}

fn state_conditions_from_bytes(default_state_address: u16, source: &[u8]) -> Vec<StateCondition> {
    let condition = u16::from_le_bytes([source[0], source[1]]);

    match condition {
        // Terminator.
        0xE5E6 => {
            return Vec::from([StateCondition {
                condition,
                parameter: None,
                state_address: default_state_address + 2,
            }])
        }
        // Two bytes parameter.
        0xE5EB => {
            let mut states = state_conditions_from_bytes(default_state_address + 6, &source[6..]);
            states.push(StateCondition {
                condition,
                parameter: Some(u16::from_le_bytes([source[2], source[3]])),
                state_address: u16::from_le_bytes([source[4], source[5]]),
            });
            return states;
        }
        // One byte parameter.
        0xE612 | 0xE629 => {
            let mut states = state_conditions_from_bytes(default_state_address + 5, &source[5..]);
            states.push(StateCondition {
                condition,
                parameter: Some(source[2] as u16),
                state_address: u16::from_le_bytes([source[3], source[4]]),
            });
            return states;
        }
        // No parameter.
        _ => {
            let mut states = state_conditions_from_bytes(default_state_address + 4, &source[4..]);
            states.push(StateCondition {
                condition,
                parameter: None,
                state_address: u16::from_le_bytes([source[2], source[3]]),
            });
            return states;
        }
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Load a State Condition from bytes.
    #[test]
    fn load_state_condition_from_bytes() {
        #[rustfmt::skip]
        let data = [
            0xCB, 0xED, // State Condition
            0xBB, 0xAA, // State Address
            0x29, 0xE6, // State Condition
            0xF0,       // Parameter (One Byte)
            0x77, 0x66, // State Address
            0x12, 0xE6, // State Condition
            0x42,       // Parameter (One Byte)
            0x55, 0x44, // State Address
            0xEB, 0xE5, // State Condition
            0xCD, 0xAB, // Parameter (Two Byte)
            0x33, 0x22, // State Address
            0xE6, 0xE5, // State Condition (Terminator)
            0x86,       // State Address
        ];

        let expected_state_conditions = vec![
            StateCondition {
                condition: 0xE5E6,
                parameter: None,
                state_address: 38,
            },
            StateCondition {
                condition: 0xE5EB,
                parameter: Some(0xABCD),
                state_address: 0x2233,
            },
            StateCondition {
                condition: 0xE612,
                parameter: Some(0x42),
                state_address: 0x4455,
            },
            StateCondition {
                condition: 0xE629,
                parameter: Some(0xF0),
                state_address: 0x6677,
            },
            StateCondition {
                condition: 0xEDCB,
                parameter: None,
                state_address: 0xAABB,
            },
        ];

        assert_eq!(
            state_conditions_from_bytes(16, &data),
            expected_state_conditions
        );
    }

    /// Load a Room from bytes.
    #[test]
    fn load_room_from_bytes() {
        let data = [
            0x12, // index
            0x34, // area
            0x56, // map position.0
            0x78, // map position.1
            0x9A, // width
            0xBC, // height
            0xDE, // up_scroller
            0xED, // down_scroller
            0xFF, // cre_bits
            0x34, 0x12, // doors
            0xE6, 0xE5, // state condition terminator
        ];

        #[rustfmt::skip]
        let expected_room = Room {
            index:         0x12,
            area:          0x34,
            map_position: (0x56, 0x78),
            width:         0x9A,
            height:        0xBC,
            up_scroller:   0xDE,
            down_scroller: 0xED,
            cre_bitset:    0xFF,
            doors:         0x1234,
            state_conditions: vec![StateCondition{
                                        condition: 0xE5E6,
                                        parameter: None,
                                        state_address: 20
                                    }
                                ],
        };

        assert_eq!(from_bytes(7, &data), expected_room);
    }
}
