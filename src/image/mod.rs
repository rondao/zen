use image::{Rgb, RgbImage};

use crate::{
    graphics::{
        gfx::{Gfx, Tile8},
        Palette, Rgb888,
    },
    super_metroid::{
        level_data::{Block, LevelData},
        tileset::TileTable,
        SuperMetroid,
    },
};

impl From<&Palette> for RgbImage {
    fn from(item: &Palette) -> Self {
        let mut palette_colors = item.to_colors().into_iter();

        let mut img: RgbImage = RgbImage::new(16, 16);
        for y in 0..16 {
            for x in 0..16 {
                let color = palette_colors.next().unwrap();
                img.put_pixel(x, y, Rgb([color.r, color.g, color.b]));
            }
        }
        img
    }
}

impl Tile8 {
    pub fn draw(
        &self,
        image: &mut RgbImage,
        origin: (usize, usize),
        flip: (bool, bool),
        palette: &Palette,
        sub_palette: usize,
    ) {
        for tpx in 0..8 {
            for tpy in 0..8 {
                let idx_color = self.colors[tpx + tpy * 8] as usize;
                let color: Rgb888 = palette.sub_palettes[sub_palette].colors[idx_color].into();

                // Index 0 is used for transparency.
                if idx_color != 0 {
                    image.put_pixel(
                        (origin.0 + if flip.0 { 7 - tpx } else { tpx }) as u32,
                        (origin.1 + if flip.1 { 7 - tpy } else { tpy }) as u32,
                        Rgb([color.r, color.g, color.b]),
                    );
                }
            }
        }
    }
}

impl Gfx {
    pub fn to_image(&self, palette: &Palette, sub_palette: usize) -> RgbImage {
        let mut img: RgbImage = RgbImage::new(16 * 8, self.tiles.len() as u32);
        for (tile_num, tile) in self.tiles.iter().enumerate() {
            // Position of the Tile8 we are drawing.
            let tx = (tile_num % 16) * 8;
            let ty = (tile_num / 16) * 8;

            tile.draw(&mut img, (tx, ty), (false, false), palette, sub_palette);
        }
        img
    }
}

pub fn tileset_to_image(tileset: &TileTable, palette: &Palette, graphics: &Gfx) -> RgbImage {
    let mut img: RgbImage = RgbImage::new(16 * 32, 16 * 32);

    let mut tiles = tileset.iter();
    for ty in 0..32 {
        for tx in 0..32 {
            // Each tile is composed of 4 smaller 'tile8'.
            for t in 0..4 {
                let tile = tiles.next().unwrap();
                let tile8 = &graphics.tiles[tile.gfx_index as usize];
                tile8.draw(
                    &mut img,
                    (tx * 16 + (t % 2) * 8, ty * 16 + (t / 2) * 8),
                    (tile.x_flip, tile.y_flip),
                    palette,
                    tile.sub_palette as usize,
                );
            }
        }
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
                    tile8.draw(
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
                .get(room.state_conditions[state].state_address as usize)
            {
                if let Some(level_data) = self.levels.get(state.level_address as usize) {
                    let tileset = self.tilesets[state.tileset as usize];

                    let tile_table = if tileset.use_cre {
                        self.tile_table_with_cre(tileset.tile_table)
                    } else {
                        self.tile_tables[tileset.tile_table].clone()
                    };
                    let graphics = if tileset.use_cre {
                        self.gfx_with_cre(tileset.graphic)
                    } else {
                        self.graphics[tileset.graphic].clone()
                    };

                    return Some(level_data.to_image(
                        (room.width as usize, room.height as usize),
                        &tile_table,
                        &self.palettes[tileset.palette],
                        &graphics,
                    ));
                }
            }
        }
        None
    }
}
