pub mod gfx;
pub mod palette;

pub use gfx::Gfx;

pub use palette::Bgr555;
pub use palette::Palette;
pub use palette::Rgb888;
pub use palette::SubPalette;

#[derive(Debug, Default, Clone, Copy)]
pub struct IndexedColor {
    pub index: usize,
    pub sub_palette: usize,
}
