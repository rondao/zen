use image::{Rgb, RgbImage};

use crate::{
    graphics::{
        gfx::{Gfx, TileGfx, GFX_TILE_WIDTH, TILE_SIZE},
        palette::{COLORS_BY_SUB_PALETTE, NUMBER_OF_SUB_PALETTES},
        Palette, Rgb888,
    },
    super_metroid::{
        level_data::{Block, LevelData},
        tile_table::{TileTable, TILE_TABLE_SIZE},
        tileset::{tileset_to_colors, TILESET_BLOCK_SIZE},
        SuperMetroid,
    },
};

impl From<&Palette> for RgbImage {
    fn from(item: &Palette) -> Self {
        let mut palette_colors = item.to_colors().into_iter();

        let mut img: RgbImage =
            RgbImage::new(NUMBER_OF_SUB_PALETTES as u32, COLORS_BY_SUB_PALETTE as u32);
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
    let mut img: RgbImage = RgbImage::new(
        (TILESET_BLOCK_SIZE * TILE_TABLE_SIZE) as u32,
        (TILESET_BLOCK_SIZE * TILE_TABLE_SIZE) as u32,
    );
    for (color_number, color) in tileset_to_colors(tile_table, palette, graphics)
        .iter()
        .enumerate()
    {
        img.put_pixel(
            (color_number % (TILE_TABLE_SIZE * TILESET_BLOCK_SIZE)) as u32,
            (color_number / (TILE_TABLE_SIZE * TILESET_BLOCK_SIZE)) as u32,
            Rgb([color.r, color.g, color.b]),
        );
    }
    img
}

impl LevelData {
    pub fn to_image(
        &self,
        size: (usize, usize),
        tile_table: &TileTable,
        palette: &Palette,
        graphics: &Gfx,
    ) -> RgbImage {
        let mut image: RgbImage = RgbImage::new(16 * 16 * size.0 as u32, 16 * 16 * size.1 as u32);
        if let Some(layer2) = &self.layer2 {
            self.layer_to_image(
                &mut image,
                size,
                &mut layer2.iter(),
                tile_table,
                palette,
                graphics,
            );
        }
        self.layer_to_image(
            &mut image,
            size,
            &mut self.layer1.iter(),
            tile_table,
            palette,
            graphics,
        );

        image
    }

    fn layer_to_image<'a>(
        &self,
        image: &mut RgbImage,
        size: (usize, usize),
        blocks: &mut impl Iterator<Item = &'a Block>,
        tile_table: &TileTable,
        palette: &Palette,
        graphics: &Gfx,
    ) {
        for index in 0..(16 * size.0 as usize * 16 * size.1 as usize) {
            if let Some(block) = blocks.next() {
                let tileset_tile = block.block_number as usize * 4;
                let mut tiles: Vec<_> = tile_table[tileset_tile..tileset_tile + 4]
                    .iter()
                    .copied()
                    .collect();

                if block.x_flip {
                    tiles.swap(0, 1);
                    tiles.swap(2, 3);
                }
                if block.y_flip {
                    tiles.swap(0, 2);
                    tiles.swap(1, 3);
                }

                // Each block is composed of 4 smaller 'tile8'.
                for t in 0..4 {
                    let tile = tiles[t];
                    let tile8 = &graphics.tiles[tile.gfx_index as usize];
                    tile8.to_image(
                        image,
                        (
                            (index % (size.0 * 16)) * 16 + (t % 2) * 8,
                            (index / (size.0 * 16)) * 16 + (t / 2) * 8,
                        ),
                        (tile.x_flip ^ block.x_flip, tile.y_flip ^ block.y_flip),
                        palette,
                        tile.sub_palette as usize,
                    );
                }
            }
        }
    }
}

impl SuperMetroid {
    pub fn room_to_image(&self, room_address: usize, state: usize) -> Option<RgbImage> {
        if let Some(room) = self.rooms.get(&room_address) {
            if let Some(state) = self
                .states
                .get(&(room.state_conditions[state].state_address as usize))
            {
                if let Some(level_data) = self.levels.get(&(state.level_address as usize)) {
                    let tileset = self.tilesets[state.tileset as usize];

                    let tile_table = if tileset.use_cre {
                        self.tile_table_with_cre(tileset.tile_table as usize)
                    } else {
                        self.tile_tables[&(tileset.tile_table as usize)].clone()
                    };
                    let graphics = if tileset.use_cre {
                        self.gfx_with_cre(tileset.graphic as usize)
                    } else {
                        self.graphics[&(tileset.graphic as usize)].clone()
                    };

                    return Some(level_data.to_image(
                        (room.width as usize, room.height as usize),
                        &tile_table,
                        &self.palettes[&(tileset.palette as usize)],
                        &graphics,
                    ));
                }
            }
        }
        None
    }
}
