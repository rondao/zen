use std::collections::HashMap;

pub type DoorList = Vec<u16>;

//https://wiki.metroidconstruction.com/doku.php?id=super:technical_information:data_structures#door_list
#[derive(Debug, Default, Clone)]
pub struct Doors {
    doors: HashMap<u16, DoorList>,
}

impl Doors {
    pub fn load_bytes(&mut self, doorlist_address: u16, number_of_doors: usize, source: &[u8]) {
        if let None = self.doors.get(&doorlist_address) {
            let door_list = source[..number_of_doors * 2]
                .chunks(2)
                .map(|bytes| u16::from_le_bytes([bytes[0], bytes[1]]))
                .collect();
            self.doors.insert(doorlist_address, door_list);
        }
    }

    pub fn get_doors(&self, doorlist_address: u16) -> &DoorList {
        self.doors.get(&doorlist_address).unwrap()
    }
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
