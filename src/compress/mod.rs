pub mod lz5_compress;
pub mod lz5_decompress;

pub use lz5_compress::compress as lz5_compress;
pub use lz5_decompress::decompress as lz5_decompress;

use std::{error::Error, fmt};

pub struct Lz5Error;

impl Error for Lz5Error {}

impl fmt::Display for Lz5Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Failed to decompress data using LZ5 algorithm.")
    }
}

impl fmt::Debug for Lz5Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Failed to decompress data using LZ5 algorithm.")
    }
}
