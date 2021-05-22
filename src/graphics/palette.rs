use std::convert::TryInto;

#[derive(Debug, Default, Clone, Copy)]
pub struct Palette {
    pub sub_palettes: [SubPalette; 8],
}

impl Palette {
    pub fn from_bytes(bytes: &[u8]) -> Self {
        Palette {
            sub_palettes: bytes
                .chunks(16 * 2) // 16 Colors per sub palette.
                .map(|sub_palette_bytes| {
                    SubPalette::from_bytes(sub_palette_bytes.try_into().unwrap())
                })
                .collect::<Vec<_>>()
                .try_into()
                .unwrap(),
        }
    }

    pub fn to_colors(&self) -> Vec<Rgb888> {
        let mut color_vec = Vec::with_capacity(16 * 16);
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

#[derive(Debug, Default, Clone, Copy)]
pub struct SubPalette {
    pub colors: [Bgr555; 16],
}

impl SubPalette {
    fn from_bytes(source: &[u8; 32]) -> Self {
        SubPalette {
            colors: source
                .chunks(2) // 2 Bytes per color.
                .map(|color_bytes| Bgr555::from_bytes(color_bytes.try_into().unwrap()))
                .collect::<Vec<_>>()
                .try_into()
                .unwrap(),
        }
    }
}

#[derive(Debug, Default, Clone, Copy)]
pub struct Bgr555 {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub u: u8,
}

impl Bgr555 {
    fn from_bytes(source: &[u8; 2]) -> Self {
        let two_bytes = u16::from_le_bytes(*source);
        Self {
            u: ((two_bytes & 0b1000_0000_0000_0000) >> 15) as u8,
            b: ((two_bytes & 0b0111_1100_0000_0000) >> 10) as u8,
            g: ((two_bytes & 0b0000_0011_1110_0000) >> 05) as u8,
            r: ((two_bytes & 0b0000_0000_0001_1111) >> 00) as u8,
        }
    }
}

#[derive(Debug, Default, Clone, Copy)]
pub struct Rgb888 {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl From<&Bgr555> for Rgb888 {
    fn from(color: &Bgr555) -> Self {
        Rgb888 {
            r: color.r << 3,
            g: color.g << 3,
            b: color.b << 3,
        }
    }
}
