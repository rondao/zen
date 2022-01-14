pub type DoorList = Vec<u16>;

//https://wiki.metroidconstruction.com/doku.php?id=super:technical_information:data_structures#door_list
pub fn load_bytes(number_of_doors: usize, source: &[u8]) -> DoorList {
    source[..number_of_doors * 2]
        .chunks(2)
        .map(|bytes| u16::from_le_bytes([bytes[0], bytes[1]]))
        .collect()
}

// Workaround for finding how many doors a Room have.
// fn get_number_of_doors_on(level_data: &LevelData) -> usize {
//     let mut number_of_doors = 0;
//     for (block, bts) in level_data.layer1.iter().zip(level_data.bts.iter()) {
//         if block.block_type == 0x9 {
//             number_of_doors = number_of_doors.max(*bts + 1);
//         }
//     }
//     number_of_doors as usize
// }

mod tests {
    use super::*;

    /// Load a door list from bytes with 3 door pointers, each as a 2 byte address in Little Endian.
    #[test]
    fn load_door_list_from_bytes() {
        let data = [0x34, 0x12, 0x78, 0x56, 0xBC, 0x9A];
        let door_list = load_bytes(3, &data);

        let expected_door_list = [0x1234, 0x5678, 0x9ABC];
        assert_eq!(door_list, expected_door_list);
    }
}
