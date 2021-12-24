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
