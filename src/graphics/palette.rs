use std::convert::TryInto;

use crate::ParseError;

pub const NUMBER_OF_SUB_PALETTES: usize = 8;

#[derive(Debug, Default, Clone)]
pub struct Palette {
    pub sub_palettes: [SubPalette; NUMBER_OF_SUB_PALETTES],
}

/// Palette format reference: https://georgjz.github.io/snesaa03/
pub fn from_bytes(mut source: &[u8]) -> Result<Palette, ParseError> {
    let bytes_per_color = if source[..3] == *b"TPL" {
        // If bytes contain 'TPL' header, extract type.
        let tpl_type = source[4];
        source = &source[5..];

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
        let mut color_vec = Vec::with_capacity(NUMBER_OF_SUB_PALETTES * COLORS_BY_SUB_PALETTE);
        for sub_palette in self.sub_palettes.iter() {
            color_vec.extend(
                sub_palette
                    .colors
                    .iter()
                    .map::<Rgb888, _>(|color| color.into())
                    .into_iter(),
            );
        }
        color_vec
    }
}

pub const COLORS_BY_SUB_PALETTE: usize = 16;

#[derive(Debug, Default, Clone, Copy)]
pub struct SubPalette {
    pub colors: [Bgr555; COLORS_BY_SUB_PALETTE],
}

#[derive(Debug, Default, Clone, Copy)]
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

#[derive(Debug, Default, Clone, Copy)]
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
