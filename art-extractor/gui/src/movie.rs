use art_extractor_core::movie::Movie;
use art_extractor_core::sprite::Color;
use art_extractor_core::surface::Surface;
use ves_cache::SliceCache;
use ves_geom::SpaceUnit;
use crate::{egui, ToEgui};
use crate::egui::{ColorImage, ImageData};

struct GuiMovieFrameSprite {
    rect: egui::Rect,
    texture: egui::TextureHandle,
}

pub struct GuiMovieFrame {
    sprites: Vec<GuiMovieFrameSprite>,
}

impl GuiMovieFrame {
    /// Creates a new instance.
    ///
    /// This should normally only be called when a new movie frame is to be rendered. Otherwise this instance should be reused between
    /// renderings.
    pub fn new(ctx: &egui::Context, movie: &Movie, frame_index: usize) -> Self {
        let palettes = SliceCache::new(movie.palettes());
        let tiles = SliceCache::new(movie.tiles());
        let movie_frame = &movie.frames()[frame_index];

        let mut sprites = Vec::with_capacity(movie_frame.sprites().len());
        for sprite in movie_frame.sprites().iter().rev() {
            let palette = &palettes[sprite.palette()];
            let tile = &tiles[sprite.tile()];
            let surf = tile.surface();
            let surf_data = surf.data();

            let mut raw_image = vec![0u8; surf.data().len() * 4]; // 4 bytes per pixel (RGBA)
            let mut raw_image_idx: usize = 0;

            art_extractor_core::surface::surface_iterate(
                surf.size(), surf.size().as_rect(),
                sprite.h_flip(), sprite.v_flip(), // TODO: We can do flipping with the mesh/Image instead of in the texture (using UV)
                |_, idx| {
                    let color = &palette[surf_data[idx]];

                    let col_data = match color {
                        Color::Opaque(col) => [col.r, col.g, col.b, 0xff],
                        Color::Transparent => [0x00, 0x00, 0x00, 0x00],
                    };

                    raw_image[raw_image_idx..raw_image_idx + 4].copy_from_slice(&col_data);
                    raw_image_idx += 4;
                }).unwrap();

            let w: usize = surf.size().width.raw().try_into().unwrap();
            let h: usize = surf.size().height.raw().try_into().unwrap();
            let color_image = ColorImage::from_rgba_unmultiplied([w, h], &raw_image);

            let texture = ctx.load_texture("something", ImageData::Color(color_image));
            let rect = art_extractor_core::geom_art::Rect::new(sprite.position(), surf.size()).to_egui();

            let gui_sprite = GuiMovieFrameSprite {
                rect,
                texture,
            };

            sprites.push(gui_sprite);
        }

        Self { sprites }
    }

    pub fn show(&self, ui: &mut egui::Ui) {
        // TODO: The scaling is not pixel-perfect, probably something to do with the rendering phase. It might be configurable.
        let zoom: f32 = 2.0;

        let from_rect = egui::Rect::from_min_size(egui::Pos2::ZERO, ui.available_size());
        let to_rect = egui::Rect::from_min_size(egui::Pos2::ZERO, zoom * ui.available_size());
        let transform = egui::emath::RectTransform::from_to(from_rect, to_rect);

        for sprite in &self.sprites {
            let rect = transform.transform_rect(sprite.rect);
            let image = egui::Image::new(&sprite.texture, rect.size());
            ui.put(rect, image);
        }
    }
}
