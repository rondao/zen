// Reference: https://en.m.wikibooks.org/wiki/Super_NES_Programming/SNES_memory_map
#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub struct LoRom {
    pub address: usize,
}

#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub struct Pc {
    pub address: usize,
}

impl From<LoRom> for Pc {
    fn from(lo_rom: LoRom) -> Self {
        // LoRom address from bank $00-$7F is mirrored to $80-$FF.
        let address = lo_rom.address & !0x80_0000;

        let number_of_banks = (address & 0xFF_0000) / 0x01_0000;

        // For each bank, from offset $0000-$7FFF is SNES reserved, and only $8000-$FFFF is ROM address.
        let offset = address & 0x00_FFFF & !0x8000;

        Pc {
            address: number_of_banks * 0x8000 + offset,
        }
    }
}

impl From<Pc> for LoRom {
    fn from(pc: Pc) -> Self {
        // Each $8000 address in Pc is a bank in LoRom.
        let number_of_banks = pc.address / 0x8000;

        // Less then $8000 will be an offset after $8000 in the LoRom.
        let offset = pc.address % 0x00_8000;

        LoRom {
            address: 0x80_0000 + (number_of_banks * 0x01_0000) + 0x8000 + offset,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Load a single 8x8 tile from bytes with 4 bits per plane.
    #[test]
    fn convert_lo_rom_and_pc_address_betweem_themselves() {
        #[rustfmt::skip]
        let addresses = [
            // Both 'LoRom' 00:8000 and 80:8000 translate to 'Pc' 00:0000, but 'Pc' to 'LoRom' is always 80:8000.
            // Original LoRom Address     Pc Address                Expected Pc to LoRom Address
            (LoRom { address: 0x008000 }, Pc { address: 0x000000 }, LoRom { address: 0x808000 }),
            (LoRom { address: 0x07ABDF }, Pc { address: 0x03ABDF }, LoRom { address: 0x87ABDF }),
            (LoRom { address: 0x128897 }, Pc { address: 0x090897 }, LoRom { address: 0x928897 }),
            (LoRom { address: 0x5C0DD7 }, Pc { address: 0x2E0DD7 }, LoRom { address: 0xDC8DD7 }),
            (LoRom { address: 0x692DE8 }, Pc { address: 0x34ADE8 }, LoRom { address: 0xE9ADE8 }),
            (LoRom { address: 0x808000 }, Pc { address: 0x000000 }, LoRom { address: 0x808000 }),
            (LoRom { address: 0x97ED8A }, Pc { address: 0x0BED8A }, LoRom { address: 0x97ED8A }),
            (LoRom { address: 0xA8A2C4 }, Pc { address: 0x1422C4 }, LoRom { address: 0xA8A2C4 }),
            (LoRom { address: 0xB0948B }, Pc { address: 0x18148B }, LoRom { address: 0xB0948B }),
            (LoRom { address: 0xF94A53 }, Pc { address: 0x3CCA53 }, LoRom { address: 0xF9CA53 }),
            (LoRom { address: 0xFFFFFF }, Pc { address: 0x3FFFFF }, LoRom { address: 0xFFFFFF }),
        ];

        for (original_lo_rom, pc, expected_lo_rom) in addresses {
            assert_eq!(Pc::from(original_lo_rom), pc);
            assert_eq!(LoRom::from(pc), expected_lo_rom);
        }
    }
}
