use address::Pc;

pub mod address;
pub mod compress;
pub mod graphics;
pub mod image;
pub mod super_metroid;

pub struct Rom {
    pub rom: Vec<u8>,
}

impl Rom {
    pub fn read(&self, start: Pc, end: Pc) -> &[u8] {
        &self.rom[start.address..end.address]
    }
}

pub struct ParseError;

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Failed to parse bytes into data.")
    }
}

impl std::fmt::Debug for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Failed to parse bytes into data.")
    }
}

impl std::error::Error for ParseError {}
