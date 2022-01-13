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

mod tests {
    /// Test the Lz5 decompression.
    #[test]
    fn decompress_files_with_lz5() {
        let test_dir = "/home/rondao/dev/snes_data/test";
        let test_cases = [
            "BAC629.gfx",
            "BEE78D.gfx",
            "C1B6F6.ttb",
            "C2AD7C.tpl",
            "C2B5E4.tpl",
            "C2C2BB.lvl",
            "C2855F.ttb",
            "CC82A8.lvl",
        ];

        for filename in test_cases {
            let decompressed_data = super::lz5_decompress(
                &std::fs::read(format!("{}/{}.bin", test_dir, filename)).unwrap(),
            )
            .unwrap();
            let expected_data = std::fs::read(format!("{}/{}", test_dir, filename)).unwrap();
            assert_eq!(decompressed_data, expected_data);
        }
    }

    /// Test the Lz5 compression.
    /// Certify that we can obtain the same data by decompressing it.
    #[test]
    fn compress_files_with_lz5() {
        let test_dir = "/home/rondao/dev/snes_data/test";
        let test_cases = [
            "BAC629.gfx",
            "BEE78D.gfx",
            "C1B6F6.ttb",
            "C2AD7C.tpl",
            "C2B5E4.tpl",
            "C2C2BB.lvl",
            "C2855F.ttb",
            "CC82A8.lvl",
        ];

        for filename in test_cases {
            let data = std::fs::read(format!("{}/{}", test_dir, filename)).unwrap();

            let compressed_data = super::lz5_compress(&data);
            let decompressed_data = super::lz5_decompress(&compressed_data).unwrap();

            assert_eq!(decompressed_data, data);
        }
    }
}
