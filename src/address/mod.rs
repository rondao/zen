// Reference: https://en.m.wikibooks.org/wiki/Super_NES_Programming/SNES_memory_map
#[derive(Debug, Default, Clone, Copy)]
pub struct LoRom {
    pub address: usize,
}

#[derive(Debug, Default, Clone, Copy)]
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
