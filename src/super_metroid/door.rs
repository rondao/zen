/// Door format reference: https://wiki.metroidconstruction.com/doku.php?id=super:technical_information:data_structures#door_header
#[derive(Debug, Default, Clone, PartialEq)]
pub struct Door {
    pub destination_room: u16, // Destination room header pointer (bank $8F)
    pub elevator_property: u8,
    pub orientation: u8,
    pub x_low_byte: u8,
    pub y_low_byte: u8,
    pub x_high_byte: u8,
    pub y_high_byte: u8,
    pub samus_door_distance: u16, // Distance from door to spawn Samus
    pub custom_asm: u16,          // Custom door ASM to execute (bank $8F)
}

pub const DOOR_BYTE_SIZE: usize = 12;

#[rustfmt::skip]
pub fn load_bytes(number_of_doors: usize, source: &[u8]) -> Vec<Door> {
    source[..number_of_doors * 12]
        .chunks(12)
        .map(|bytes| Door {
            destination_room:    u16::from_le_bytes([bytes[0], bytes[1]]),
            elevator_property:   bytes[2],
            orientation:         bytes[3],
            x_low_byte:          bytes[4],
            y_low_byte:          bytes[5],
            x_high_byte:         bytes[6],
            y_high_byte:         bytes[7],
            samus_door_distance: u16::from_le_bytes([bytes[8], bytes[9]]),
            custom_asm:          u16::from_le_bytes([bytes[10], bytes[11]]),
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Load 3 doors from bytes.
    #[test]
    fn load_doors_from_bytes() {
        #[rustfmt::skip]
        let data = [0xF8, 0x91, 0x00, 0x03, 0x00, 0x00, 0x04, 0x00, 0x00, 0x80, 0x00, 0x00,
                    0xF8, 0x91, 0x01, 0x03, 0x00, 0x00, 0x04, 0x02, 0xF0, 0x80, 0xF0, 0x00,
                    0xFD, 0x92, 0x02, 0x05, 0x4E, 0x06, 0x04, 0x00, 0x0F, 0x80, 0x0F, 0xF0];
        let doors = load_bytes(3, &data);

        let expected_doors = [
            Door {
                destination_room: 0x91F8,
                elevator_property: 0x00,
                orientation: 0x03,
                x_low_byte: 0x00,
                y_low_byte: 0x00,
                x_high_byte: 0x04,
                y_high_byte: 0x00,
                samus_door_distance: 0x8000,
                custom_asm: 0x0000,
            },
            Door {
                destination_room: 0x91F8,
                elevator_property: 0x01,
                orientation: 0x03,
                x_low_byte: 0x00,
                y_low_byte: 0x00,
                x_high_byte: 0x04,
                y_high_byte: 0x02,
                samus_door_distance: 0x80F0,
                custom_asm: 0x00F0,
            },
            Door {
                destination_room: 0x92FD,
                elevator_property: 0x02,
                orientation: 0x05,
                x_low_byte: 0x4E,
                y_low_byte: 0x06,
                x_high_byte: 0x04,
                y_high_byte: 0x00,
                samus_door_distance: 0x800F,
                custom_asm: 0xF00F,
            },
        ];
        assert_eq!(doors, expected_doors);
    }
}
