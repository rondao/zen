use std::convert::TryInto;

use crate::ParseError;

pub const NUMBER_OF_SUB_PALETTES: usize = 8;

#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub struct Palette {
    pub sub_palettes: [SubPalette; NUMBER_OF_SUB_PALETTES],
}

/// Palette format reference: https://georgjz.github.io/snesaa03/
pub fn from_bytes(mut source: &[u8]) -> Result<Palette, ParseError> {
    let bytes_per_color = if source[..3] == *b"TPL" {
        // If bytes contain 'TPL' header, extract type.
        let tpl_type = source[3];
        source = &source[4..];

        match tpl_type {
            0x00 => 3,
            0x02 => 2,
            _ => return Err(ParseError),
        }
    } else {
        2 // SNES Default.
    };

    let sub_palettes: Result<Vec<_>, _> = source
        .chunks(COLORS_BY_SUB_PALETTE * bytes_per_color)
        .map(|sub_palette_bytes| {
            // Try creating a sub palette.
            let colors: Result<Vec<_>, _> = sub_palette_bytes
                .chunks(bytes_per_color)
                .map(|color_bytes| Bgr555::from_bytes(color_bytes))
                .collect();

            Ok(SubPalette {
                colors: colors?.try_into().unwrap(),
            })
        })
        .collect();

    let mut sub_palettes = sub_palettes?;
    if sub_palettes.len() < NUMBER_OF_SUB_PALETTES {
        sub_palettes.extend(vec![
            SubPalette::default();
            NUMBER_OF_SUB_PALETTES - sub_palettes.len()
        ]);
    }

    if sub_palettes.len() <= NUMBER_OF_SUB_PALETTES {
        Ok(Palette {
            sub_palettes: sub_palettes.try_into().unwrap(),
        })
    } else {
        // Palette need exact NUMBER_OF_SUB_PALETTES subpalettes.
        Err(ParseError)
    }
}

impl Palette {
    pub fn to_colors(&self) -> Vec<Rgb888> {
        let mut colors = Vec::with_capacity(NUMBER_OF_SUB_PALETTES * COLORS_BY_SUB_PALETTE);
        for sub_palette in self.sub_palettes.iter() {
            colors.extend(
                sub_palette
                    .colors
                    .iter()
                    .map::<Rgb888, _>(|color| color.into())
                    .into_iter(),
            );
        }
        colors
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(NUMBER_OF_SUB_PALETTES * COLORS_BY_SUB_PALETTE * 2);
        for sub_palette in self.sub_palettes.iter() {
            bytes.extend(
                sub_palette
                    .colors
                    .iter()
                    .fold(Vec::new(), |mut accum, value| {
                        accum.extend(value.to_bytes());
                        accum
                    }),
            );
        }
        bytes
    }
}

pub const COLORS_BY_SUB_PALETTE: usize = 16;

#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub struct SubPalette {
    pub colors: [Bgr555; COLORS_BY_SUB_PALETTE],
}

#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub struct Bgr555 {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub u: u8,
}

impl Bgr555 {
    /// Palette format reference: https://georgjz.github.io/snesaa03/
    fn from_bytes(source: &[u8]) -> Result<Self, ParseError> {
        match source.len() {
            2 => {
                let two_bytes = u16::from_le_bytes(source.try_into().unwrap());
                Ok(Self {
                    u: ((two_bytes & 0b1000_0000_0000_0000) >> 15) as u8,
                    b: ((two_bytes & 0b0111_1100_0000_0000) >> 10) as u8,
                    g: ((two_bytes & 0b0000_0011_1110_0000) >> 05) as u8,
                    r: ((two_bytes & 0b0000_0000_0001_1111) >> 00) as u8,
                })
            }
            3 => Ok(Self {
                u: 0,
                r: source[0] >> 3 as u8,
                g: source[1] >> 3 as u8,
                b: source[2] >> 3 as u8,
            }),
            _ => Err(ParseError),
        }
    }

    fn to_bytes(&self) -> [u8; 2] {
        let high_byte = (self.u << 7) + (self.b << 2) + ((self.g & 0b11000) >> 3);
        let low_byte = ((self.g & 0b111) << 5) + self.r;
        [low_byte, high_byte] // Little Endian.
    }
}

impl From<Rgb888> for Bgr555 {
    fn from(color: Rgb888) -> Self {
        Self {
            r: color.r >> 3,
            g: color.g >> 3,
            b: color.b >> 3,
            u: 0,
        }
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub struct Rgb888 {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl From<Bgr555> for Rgb888 {
    fn from(color: Bgr555) -> Self {
        (&color).into()
    }
}

impl From<&Bgr555> for Rgb888 {
    fn from(color: &Bgr555) -> Self {
        Self {
            r: color.r << 3,
            g: color.g << 3,
            b: color.b << 3,
        }
    }
}

#[cfg(test)]
mod tests {
    use std::error::Error;

    use super::*;

    /// Load a palette from bytes with bgr555 format in two bytes.
    /// Convert the palette back into bytes.
    /// Test done with headered and unheadered bytes.
    /// Palette format: [NUMBER_OF_SUB_PALETTES * [COLORS_BY_SUB_PALETTE * [Bgr555]]]
    #[test]
    fn convert_palette_from_and_into_two_bytes_bgr555_format() -> Result<(), Box<dyn Error>> {
        let sub_palette: [Bgr555; COLORS_BY_SUB_PALETTE] = [Bgr555 {
            r: 4,
            g: 10,
            b: 21,
            u: 0,
        }; COLORS_BY_SUB_PALETTE];

        let sub_palettes: [SubPalette; NUMBER_OF_SUB_PALETTES] = [SubPalette {
            colors: sub_palette,
        }; NUMBER_OF_SUB_PALETTES];

        let expected_palette = Palette { sub_palettes };
        let palette_bytes = [0b010_00100, 0b0_10101_01] // Bgr555 { r: 4, g: 10, b: 21, u: 0 }
            .repeat(COLORS_BY_SUB_PALETTE)
            .repeat(NUMBER_OF_SUB_PALETTES);

        let palette = from_bytes(&palette_bytes)?;
        assert_eq!(palette, expected_palette);
        assert_eq!(palette.to_bytes(), palette_bytes);

        let mut palette_bytes_with_header_for_two_bytes = b"TPL".to_vec(); // Header.
        palette_bytes_with_header_for_two_bytes.push(0x02); // Two byte format.
        palette_bytes_with_header_for_two_bytes.extend(&palette_bytes); //Data.

        let palette = from_bytes(&palette_bytes_with_header_for_two_bytes)?;
        assert_eq!(palette, expected_palette);
        assert_eq!(palette.to_bytes(), palette_bytes);

        Ok(())
    }

    /// Load a palette from bytes with bgr555 in two bytes format, but with missing colors.
    /// Expected to load the missing colors with default bgr555 colors.
    #[test]
    fn load_palette_from_two_bytes_bgr555_format_with_missing_sub_palettes(
    ) -> Result<(), Box<dyn Error>> {
        let sub_palette: [Bgr555; COLORS_BY_SUB_PALETTE] = [Bgr555 {
            r: 4,
            g: 10,
            b: 21,
            u: 0,
        }; COLORS_BY_SUB_PALETTE];

        let sub_palettes: [SubPalette; NUMBER_OF_SUB_PALETTES] = [
            SubPalette {
                colors: sub_palette,
            },
            SubPalette {
                colors: sub_palette,
            },
            SubPalette {
                colors: sub_palette,
            },
            SubPalette::default(),
            SubPalette::default(),
            SubPalette::default(),
            SubPalette::default(),
            SubPalette::default(),
        ];

        let expected_palette = Palette { sub_palettes };
        let palette_bytes = [0b010_00100, 0b0_10101_01] // Bgr555 { r: 4, g: 10, b: 21, u: 0 }
            .repeat(COLORS_BY_SUB_PALETTE)
            .repeat(3);

        let palette = from_bytes(&palette_bytes)?;
        assert_eq!(palette, expected_palette);

        Ok(())
    }

    /// Load a palette from bytes with bgr555 in two bytes format, but with exceeding number of sub-palettes.
    /// Expected to return an error.
    #[test]
    fn load_palette_from_two_bytes_bgr555_format_with_exceeding_sub_palettes() {
        let palette_bytes = [0b010_00100, 0b0_10101_01] // Bgr555 { r: 4, g: 10, b: 21, u: 0 }
            .repeat(COLORS_BY_SUB_PALETTE)
            .repeat(NUMBER_OF_SUB_PALETTES + 1);
        assert!(from_bytes(&palette_bytes).is_err());
    }

    /// The types Bgr555 and Rgb888 should be convertable bewteen themselves.
    #[test]
    fn convert_bgr555_and_rgb888_between_themselves() {
        let bgr555_and_rgb888_colors = [
            (
                // Bgr555 { r: 0, g: 0, b: 0, u: 0 }
                Bgr555 {
                    r: 0b00000,
                    g: 0b00000,
                    b: 0b00000,
                    u: 0,
                },
                // Rgb888 { r: 0, g: 0, b: 0 }
                Rgb888 {
                    r: 0b00000000,
                    g: 0b00000000,
                    b: 0b00000000,
                },
            ),
            (
                // Bgr555 { r: 23, g: 27, b: 29, u: 0 }
                Bgr555 {
                    r: 0b10111,
                    g: 0b11011,
                    b: 0b11101,
                    u: 0,
                },
                // Rgb888 { r: 184, g: 216, b: 232 }
                Rgb888 {
                    r: 0b10111000,
                    g: 0b11011000,
                    b: 0b11101000,
                },
            ),
            (
                // Bgr555 { r: r: 10, g: 21, b: 7, u: 0 }
                Bgr555 {
                    r: 0b01010,
                    g: 0b10101,
                    b: 0b00111,
                    u: 0,
                },
                // Rgb888 { r: 80, g: 84, b: 56 }
                Rgb888 {
                    r: 0b01010000,
                    g: 0b10101000,
                    b: 0b00111000,
                },
            ),
            (
                // Bgr555 { r: 31, g: 31, b: 31, u: 0 }
                Bgr555 {
                    r: 0b11111,
                    g: 0b11111,
                    b: 0b11111,
                    u: 0,
                },
                // Rgb888 { r: 248, g: 248, b: 248 }
                Rgb888 {
                    r: 0b11111000,
                    g: 0b11111000,
                    b: 0b11111000,
                },
            ),
        ];

        for (bgr555, rgb888) in bgr555_and_rgb888_colors {
            assert_eq!(bgr555, Bgr555::from(rgb888));
            assert_eq!(rgb888, Rgb888::from(bgr555));
        }
    }

    /// Load a color in Bgr555 format from two bytes.
    /// Convert the Bgr555 type format into two bytes.
    /// Expected byte format: [UBBB_BBGG, GGGR_RRRR]
    #[test]
    fn convert_bgr555_from_and_into_two_bytes() -> Result<(), Box<dyn Error>> {
        // Bytes are in Little-Endian.
        let bytes_and_expected_bgr555 = [
            (
                // Bgr555 { r: 0, g: 0, b: 0, u: 0 }
                Bgr555 {
                    r: 0b00000,
                    g: 0b00000,
                    b: 0b00000,
                    u: 0,
                },
                [0b0_00000_00, 0b000_00000],
            ),
            (
                // Bgr555 { r: 23, g: 27, b: 29, u: 1 }
                Bgr555 {
                    r: 0b10111,
                    g: 0b11011,
                    b: 0b11101,
                    u: 1,
                },
                [0b011_10111, 0b1_11101_11],
            ),
            (
                // Bgr555 { r: 10, g: 21, b: 7, u: 0 }
                Bgr555 {
                    r: 0b01010,
                    g: 0b10101,
                    b: 0b00111,
                    u: 0,
                },
                [0b1_01010_10, 0b000_11110],
            ),
            (
                // Bgr555 { r: 31, g: 31, b: 31, u: 1 }
                Bgr555 {
                    r: 0b11111,
                    g: 0b11111,
                    b: 0b11111,
                    u: 1,
                },
                [0b1_11111_11, 0b111_11111],
            ),
        ];

        for (bgr555, bytes) in bytes_and_expected_bgr555 {
            assert_eq!(bgr555, Bgr555::from_bytes(&bytes)?);
            assert_eq!(bytes, bgr555.to_bytes());
        }

        Ok(())
    }

    /// Load a color in Bgr555 format from three bytes.
    /// Expected byte format: [BBBB_BXXX, GGGG_GXXX, RRRR_RXXX]
    #[test]
    fn load_bgr555_from_three_bytes() -> Result<(), Box<dyn Error>> {
        // Bytes are in Little-Endian.
        let bytes_and_expected_bgr555 = [
            (
                // Bgr555 { r: 0, g: 0, b: 0, u: 0 }
                Bgr555 {
                    r: 0b00000,
                    g: 0b00000,
                    b: 0b00000,
                    u: 0,
                },
                [0b00000_000, 0b00000_000, 0b00000_000],
            ),
            (
                // Bgr555 { r: 23, g: 27, b: 29, u: 1 }
                Bgr555 {
                    r: 0b10111,
                    g: 0b11011,
                    b: 0b11101,
                    u: 0,
                },
                [0b10111_000, 0b11011_000, 0b11101_000],
            ),
            (
                // Bgr555 { r: 10, g: 21, b: 7, u: 0 }
                Bgr555 {
                    r: 0b01010,
                    g: 0b10101,
                    b: 0b00111,
                    u: 0,
                },
                [0b01010_000, 0b10101_000, 0b00111_000],
            ),
            (
                // Bgr555 { r: 31, g: 31, b: 31, u: 1 }
                Bgr555 {
                    r: 0b11111,
                    g: 0b11111,
                    b: 0b11111,
                    u: 0,
                },
                [0b11111_000, 0b11111_000, 0b11111_000],
            ),
        ];

        for (bgr555, bytes) in bytes_and_expected_bgr555 {
            assert_eq!(bgr555, Bgr555::from_bytes(&bytes)?);
        }

        Ok(())
    }

    /// Fail loading a color in Bgr555 format from incorred sized bytes.
    /// Expected format: [BBBB_BXXX, GGGG_GXXX, RRRR_RXXX]
    #[test]
    fn fail_to_load_bgr555_from_incorrect_sized_bytes() {
        assert!(Bgr555::from_bytes(&[]).is_err());
        assert!(Bgr555::from_bytes(&[0x00]).is_err());
        assert!(Bgr555::from_bytes(&[0x00, 0x00, 0x00, 0x00]).is_err());
    }
}
