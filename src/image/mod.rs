use image::{Rgb, RgbImage};

use crate::{
    graphics::{
        gfx::{Gfx, TileGfx, GFX_TILE_WIDTH, TILE_SIZE},
        palette::{COLORS_BY_SUB_PALETTE, NUMBER_OF_SUB_PALETTES},
        Palette, Rgb888,
    },
    super_metroid::{
        level_data::{LevelData, BLOCKS_PER_SCREEN},
        room::Room,
        state::State,
        tile_table::{TileTable, TILE_TABLE_SIZE},
        tileset::{tileset_to_colors, TILESET_BLOCK_SIZE},
        SuperMetroid,
    },
};

impl Palette {
    pub fn to_image(&self) -> RgbImage {
        let mut palette_colors = self.to_colors().into_iter();

        let mut img: RgbImage =
            RgbImage::new(COLORS_BY_SUB_PALETTE as u32, NUMBER_OF_SUB_PALETTES as u32);
        for y in 0..NUMBER_OF_SUB_PALETTES {
            for x in 0..COLORS_BY_SUB_PALETTE {
                let color = palette_colors.next().unwrap();
                img.put_pixel(x as u32, y as u32, Rgb([color.r, color.g, color.b]));
            }
        }
        img
    }
}

impl TileGfx {
    pub fn to_image(
        &self,
        image: &mut RgbImage,
        origin: (usize, usize),
        flip: (bool, bool),
        palette: &Palette,
        sub_palette: usize,
    ) {
        for (pixel, idx_color) in self.flip(flip).iter().enumerate() {
            // Index 0 is used for transparency.
            if *idx_color != 0 {
                let color: Rgb888 =
                    palette.sub_palettes[sub_palette].colors[*idx_color as usize].into();
                image.put_pixel(
                    (origin.0 + pixel % TILE_SIZE) as u32,
                    (origin.1 + pixel / TILE_SIZE) as u32,
                    Rgb([color.r, color.g, color.b]),
                );
            }
        }
    }
}

impl Gfx {
    pub fn to_image(&self, palette: &Palette, sub_palette: usize) -> RgbImage {
        let mut img: RgbImage = RgbImage::new(
            (GFX_TILE_WIDTH * TILE_SIZE) as u32,
            (self.tiles.len() * TILE_SIZE / GFX_TILE_WIDTH) as u32,
        );
        for (color_number, index_color) in self.to_indexed_colors().iter().enumerate() {
            let color: Rgb888 =
                palette.sub_palettes[sub_palette].colors[*index_color as usize].into();

            img.put_pixel(
                (color_number % (GFX_TILE_WIDTH * TILE_SIZE)) as u32,
                (color_number / (GFX_TILE_WIDTH * TILE_SIZE)) as u32,
                Rgb([color.r, color.g, color.b]),
            );
        }
        img
    }
}

pub fn tileset_to_image(tile_table: &TileTable, palette: &Palette, graphics: &Gfx) -> RgbImage {
    let mut image: RgbImage = RgbImage::new(
        (TILESET_BLOCK_SIZE * TILE_TABLE_SIZE) as u32,
        (TILESET_BLOCK_SIZE * TILE_TABLE_SIZE) as u32,
    );
    for (color_number, color) in tileset_to_colors(tile_table, palette, graphics)
        .iter()
        .enumerate()
    {
        image.put_pixel(
            (color_number % image.width() as usize) as u32,
            (color_number / image.width() as usize) as u32,
            Rgb([color.r, color.g, color.b]),
        );
    }
    image
}

impl LevelData {
    pub fn to_image(
        &self,
        size: (usize, usize),
        tile_table: &TileTable,
        palette: &Palette,
        graphics: &Gfx,
    ) -> RgbImage {
        let mut image: RgbImage = RgbImage::new(
            (BLOCKS_PER_SCREEN * TILE_SIZE * 2 * size.0) as u32,
            (BLOCKS_PER_SCREEN * TILE_SIZE * 2 * size.1) as u32,
        );
        for (color_number, color) in self
            .to_colors(size, tile_table, palette, graphics)
            .enumerate()
        {
            image.put_pixel(
                (color_number % image.width() as usize) as u32,
                (color_number / image.width() as usize) as u32,
                Rgb([color.r, color.g, color.b]),
            );
        }
        image
    }
}

impl SuperMetroid {
    pub fn room_to_image(&self, room: &Room, state: &State) -> RgbImage {
        let (level_data, _, palette, graphics, tile_table) = self.get_state_data(state);

        level_data.to_image(room.size(), &tile_table, &palette, &graphics)
    }
}
