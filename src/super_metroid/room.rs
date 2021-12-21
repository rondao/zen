/// Room format reference: https://wiki.metroidconstruction.com/doku.php?id=super:technical_information:data_structures#room_header
#[derive(Debug, Default, Clone)]
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

pub fn from_bytes(source: &[u8]) -> Room {
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
        doors:       ((source[9] as u16) << 8) + source[10] as u16,
        state_conditions: state_conditions_from_bytes(&source[11..]),
    };
    room
}

/// Room format reference: https://wiki.metroidconstruction.com/doku.php?id=super:technical_information:data_structures#room_header
#[derive(Debug, Default, Clone)]
pub struct StateCondition {
    pub condition: u16,
    pub parameter: Option<u16>,
    pub state_header: Option<u16>,
}

fn state_conditions_from_bytes(source: &[u8]) -> Vec<StateCondition> {
    let condition = ((source[0] as u16) << 8) + source[1] as u16;

    match condition {
        // Terminator.
        0xE5E6 => {
            return Vec::from([StateCondition {
                condition,
                parameter: None,
                state_header: None,
            }])
        }
        // Two bytes parameter.
        0xE5EB => {
            let mut states = Vec::from([StateCondition {
                condition,
                parameter: Some(((source[3] as u16) << 8) + source[4] as u16),
                state_header: Some(((source[5] as u16) << 8) + source[6] as u16),
            }]);
            states.append(&mut state_conditions_from_bytes(&source[7..]));
            return states;
        }
        // One byte parameter.
        0xE612 | 0xE629 => {
            let mut states = Vec::from([StateCondition {
                condition,
                parameter: Some(source[3] as u16),
                state_header: Some(((source[4] as u16) << 8) + source[5] as u16),
            }]);
            states.append(&mut state_conditions_from_bytes(&source[6..]));
            return states;
        }
        // No parameter.
        _ => {
            let mut states = Vec::from([StateCondition {
                condition,
                parameter: None,
                state_header: Some(((source[3] as u16) << 8) + source[4] as u16),
            }]);
            states.append(&mut state_conditions_from_bytes(&source[5..]));
            return states;
        }
    };
}
