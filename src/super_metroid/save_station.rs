/// Save Station format reference: https://patrickjohnston.org/bank/80#fC4B5
#[derive(Debug, Default, Clone, PartialEq)]
pub struct SaveStation {
    pub room_pointer: u16,
    pub door_pointer: u16,
    pub door_bts: u16,
    pub screen_x_position: u16,
    pub screen_y_position: u16,
    pub samus_y_offset: u16, // Relative to screen top.
    pub samus_x_offset: u16, // Relative to screen center.
}

impl SaveStation {
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut output = Vec::new();
        output.extend(self.room_pointer.to_le_bytes());
        output.extend(self.door_pointer.to_le_bytes());
        output.extend(self.door_bts.to_le_bytes());
        output.extend(self.screen_x_position.to_le_bytes());
        output.extend(self.screen_y_position.to_le_bytes());
        output.extend(self.samus_y_offset.to_le_bytes());
        output.extend(self.samus_x_offset.to_le_bytes());
        output
    }
}

pub const SAVE_STATION_BYTE_SIZE: usize = 14;

// Load all Save Stations from one area until the next.
// This does not load the Save Stations from the last area, which by default is the Debug ones.
pub fn load_all_from_list(
    bytes: &[u8],
    addresses: &[u8],
    number_of_areas: usize,
) -> Vec<Vec<SaveStation>> {
    let mut output = Vec::new();

    let addresses: Vec<usize> = addresses
        .chunks(2)
        .take(number_of_areas)
        .map(|address| u16::from_le_bytes([address[0], address[1]]) as usize)
        .collect();

    let mut current_area_address = addresses[0];
    for next_area_address in &addresses[1..] {
        let mut area_stations = Vec::new();
        while current_area_address != *next_area_address {
            area_stations.push(load_bytes(&bytes[current_area_address - addresses[0]..]));
            current_area_address += SAVE_STATION_BYTE_SIZE;
        }
        output.push(area_stations);
    }

    output
}

#[rustfmt::skip]
pub fn load_bytes(bytes: &[u8]) -> SaveStation {
    SaveStation {
        room_pointer:      u16::from_le_bytes([bytes[0], bytes[1]]),
        door_pointer:      u16::from_le_bytes([bytes[2], bytes[3]]),
        door_bts:          u16::from_le_bytes([bytes[4], bytes[5]]),
        screen_x_position: u16::from_le_bytes([bytes[6], bytes[7]]),
        screen_y_position: u16::from_le_bytes([bytes[8], bytes[9]]),
        samus_y_offset:    u16::from_le_bytes([bytes[10], bytes[11]]),
        samus_x_offset:    u16::from_le_bytes([bytes[12], bytes[13]]),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Load 3 Save Stations from bytes.
    #[test]
    fn load_stations_from_bytes() {
        #[rustfmt::skip]
        let data = [[0x93, 0xD5, 0x89, 0x9A, 0x00, 0x01, 0x10, 0x00, 0x01, 0x20, 0x00, 0x98, 0xFF, 0xE0],
                    [0x94, 0xCC, 0x8A, 0xBA, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xA8, 0x00, 0x00],
                    [0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x04, 0x00, 0x04, 0x00, 0x00, 0xB0, 0xAB, 0xCD],];

        #[rustfmt::skip]
        let expected_save_stations = [
            SaveStation {
                room_pointer:      0xD593,
                door_pointer:      0x9A89,
                door_bts:          0x0100,
                screen_x_position: 0x0010,
                screen_y_position: 0x2001,
                samus_y_offset:    0x9800,
                samus_x_offset:    0xE0FF,
            },
            SaveStation {
                room_pointer:      0xCC94,
                door_pointer:      0xBA8A,
                door_bts:          0x0000,
                screen_x_position: 0x0000,
                screen_y_position: 0x0000,
                samus_y_offset:    0xA800,
                samus_x_offset:    0x0000,
            },
            SaveStation {
                room_pointer:      0x0000,
                door_pointer:      0x0000,
                door_bts:          0x0000,
                screen_x_position: 0x0004,
                screen_y_position: 0x0004,
                samus_y_offset:    0xB000,
                samus_x_offset:    0xCDAB,
            },
        ];

        let save_stations = data.map(|bytes| load_bytes(&bytes));
        assert_eq!(save_stations, expected_save_stations);
    }

    /// Load 6 Save Stations divided in 3, 2 and 1 areas.
    #[test]
    fn load_all_stations_from_bytes() {
        #[rustfmt::skip]
        let data = [// 3 Save Stations from Area 0.
                    0x93, 0xD5, 0x89, 0x9A, 0x00, 0x01, 0x10, 0x00, 0x01, 0x20, 0x00, 0x98, 0xFF, 0xE0,
                    0x94, 0xCC, 0x8A, 0xBA, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xA8, 0x00, 0x00,
                    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x04, 0x00, 0x04, 0x00, 0x00, 0xB0, 0xAB, 0xCD,
                    // 2 Save Stations from Area 1.
                    0x00, 0x01, 0x20, 0x00, 0x98, 0xFF, 0xE0, 0x93, 0xD5, 0x89, 0x9A, 0x00, 0x01, 0x10,
                    0x00, 0x00, 0x00, 0x00, 0xA8, 0x00, 0x00, 0x94, 0xCC, 0x8A, 0xBA, 0x00, 0x00, 0x00,
                    // 1 Save Station from Area 2.
                    0x00, 0x04, 0x00, 0x00, 0xB0, 0xAB, 0xCD, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x04,
                    ];

        #[rustfmt::skip]
        let expected_save_stations = [
            vec![
                SaveStation {
                    room_pointer:      0xD593,
                    door_pointer:      0x9A89,
                    door_bts:          0x0100,
                    screen_x_position: 0x0010,
                    screen_y_position: 0x2001,
                    samus_y_offset:    0x9800,
                    samus_x_offset:    0xE0FF,
                },
                SaveStation {
                    room_pointer:      0xCC94,
                    door_pointer:      0xBA8A,
                    door_bts:          0x0000,
                    screen_x_position: 0x0000,
                    screen_y_position: 0x0000,
                    samus_y_offset:    0xA800,
                    samus_x_offset:    0x0000,
                },
                SaveStation {
                    room_pointer:      0x0000,
                    door_pointer:      0x0000,
                    door_bts:          0x0000,
                    screen_x_position: 0x0004,
                    screen_y_position: 0x0004,
                    samus_y_offset:    0xB000,
                    samus_x_offset:    0xCDAB,
                },
            ],
            vec![
                SaveStation {
                    room_pointer:      0x0100,
                    door_pointer:      0x0020,
                    door_bts:          0xFF98,
                    screen_x_position: 0x93E0,
                    screen_y_position: 0x89D5,
                    samus_y_offset:    0x009A,
                    samus_x_offset:    0x1001,
                },
                SaveStation {
                    room_pointer:      0x0000,
                    door_pointer:      0x0000,
                    door_bts:          0x00A8,
                    screen_x_position: 0x9400,
                    screen_y_position: 0x8ACC,
                    samus_y_offset:    0x00BA,
                    samus_x_offset:    0x0000,
                },
            ],
            vec![
                SaveStation {
                    room_pointer:      0x0400,
                    door_pointer:      0x0000,
                    door_bts:          0xABB0,
                    screen_x_position: 0x00CD,
                    screen_y_position: 0x0000,
                    samus_y_offset:    0x0000,
                    samus_x_offset:    0x0400,
                },
            ],
        ];

        let save_station_list = vec![
            (0 as u16).to_le_bytes(),
            ((SAVE_STATION_BYTE_SIZE * 3) as u16).to_le_bytes(),
            ((SAVE_STATION_BYTE_SIZE * 5) as u16).to_le_bytes(),
            ((SAVE_STATION_BYTE_SIZE * 6) as u16).to_le_bytes(),
        ];
        let save_station_list_bytes =
            save_station_list.iter().fold(Vec::new(), |mut acc, bytes| {
                acc.extend(bytes);
                acc
            });

        let save_stations = load_all_from_list(&data, &save_station_list_bytes, 4);
        assert_eq!(save_stations, expected_save_stations);
    }
}
