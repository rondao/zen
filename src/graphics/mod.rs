pub mod gfx;
pub mod palette;

use std::error::Error;
use std::fmt;

pub use palette::Bgr555;
pub use palette::Palette;
pub use palette::Rgb888;
pub use palette::SubPalette;

pub struct ParseError;

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Failed to parse bytes into data.")
    }
}

impl fmt::Debug for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Failed to parse bytes into data.")
    }
}

impl Error for ParseError {}
