use std::ops::{Index, Range};
use std::time::{Duration, Instant};
use art_extractor_core::sprite::{Color, Palette, PaletteRef, Tile, TileRef};
use art_extractor_core::surface::Surface;
use ves_cache::SliceCache;
use ves_geom::RectIntersection;
use crate::{egui, ToEgui};
use crate::egui::{ColorImage, ImageData};

struct GuiMovieFrameSprite {
    rect: art_extractor_core::geom_art::Rect,
    texture: egui::TextureHandle,
}

struct MovieFrame {
    sprites: Vec<GuiMovieFrameSprite>,
}

const ZOOM: f32 = 2.0;

impl MovieFrame {
    /// Creates a new instance.
    ///
    /// This should normally only be called when a new movie frame is to be rendered. Otherwise this instance should be reused between
    /// renderings.
    pub fn new(
        ctx: &egui::Context,
        palettes: &impl Index<PaletteRef, Output=Palette>,
        tiles: &impl Index<TileRef, Output=Tile>,
        movie_frame: &art_extractor_core::movie::MovieFrame,
    ) -> Self {
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
            let rect = art_extractor_core::geom_art::Rect::new_from_size(sprite.position(), surf.size());

            let gui_sprite = GuiMovieFrameSprite {
                rect,
                texture,
            };

            sprites.push(gui_sprite);
        }

        Self { sprites }
    }

    pub fn show(&self, ui: &mut egui::Ui, screen_size: art_extractor_core::geom_art::Size, viewport: egui::Rect) {
        // TODO: The scaling is not pixel-perfect by default. This has to do with the texture filtering in the rendering component.
        //       Currently this requires a hack in egui_glow, since there is no way for the application code to control this.
        // TODO: It seems like the UI adds spacing of an extra 8px when an image is exactly on the edge, causing the scrollbars to resize
        //       when a sprite wraps around.

        let from_rect = egui::Rect::from_min_size(egui::Pos2::ZERO, ui.available_size());
        let to_rect = egui::Rect::from_min_size(egui::pos2(-viewport.left(), -viewport.top()), ZOOM * ui.available_size());
        let transform = egui::emath::RectTransform::from_to(from_rect, to_rect);

        let intersect_pos = screen_size.as_rect().max;

        self.sprites.iter().for_each(|sprite| {
            let egui_sprite_rect = sprite.rect.to_egui();
            match sprite.rect.intersect_point(intersect_pos) {
                // No intersections; this means the sprite fits entirely on the screen
                RectIntersection::None => {
                    let rect = transform.transform_rect(egui_sprite_rect);
                    let image = egui::Image::new(&sprite.texture, rect.size());
                    ui.put(rect, image);
                }
                // Treat all other cases generically
                intersection => {
                    intersection.for_each(|rect| {
                        let egui_rect = rect.to_egui();
                        let width = egui_sprite_rect.width();
                        let height = egui_sprite_rect.height();
                        let u_x = (egui_rect.min.x - egui_sprite_rect.min.x) / width;
                        let u_y = (egui_rect.min.y - egui_sprite_rect.min.y) / height;
                        let v_x = (egui_rect.max.x - egui_sprite_rect.min.x) / width;
                        let v_y = (egui_rect.max.y - egui_sprite_rect.min.y) / height;

                        let egui_dest_rect = art_extractor_core::geom_art::Rect::new_from_size(
                            (rect.min_x() % screen_size.width, rect.min_y() % screen_size.height),
                            rect.size(),
                        ).to_egui();

                        let dest_rect = transform.transform_rect(egui_dest_rect);
                        let image = egui::Image::new(&sprite.texture, dest_rect.size())
                            .uv(egui::Rect::from_min_max(egui::pos2(u_x, u_y), egui::pos2(v_x, v_y)));

                        ui.put(dest_rect, image);
                    });
                }
            }
        });
    }
}

#[derive(Clone, Debug)]
enum PlaybackState {
    Paused,
    Playing(Instant),
}

pub struct Movie {
    movie: art_extractor_core::movie::Movie,
    frame_iter: Range<usize>,
    frame_duration: Duration,
    playback_state: PlaybackState,
    current_frame: Option<MovieFrame>,
}

impl Movie {
    pub fn new(movie: art_extractor_core::movie::Movie) -> Self {
        let frame_iter = Self::create_frame_iter(&movie);
        let frame_duration = Duration::from_secs(1) / movie.frame_rate().fps();
        Self {
            movie,
            frame_iter,
            frame_duration,
            playback_state: PlaybackState::Paused,
            current_frame: None,
        }
    }

    pub fn play(&mut self, ctx: &egui::Context, current_instant: Instant) {
        match self.playback_state {
            PlaybackState::Paused => {
                self.playback_state = PlaybackState::Playing(current_instant);
                self.next_frame(ctx);
            }
            PlaybackState::Playing(_) => {} // do nothing
        }
    }

    pub fn update(&mut self, ctx: &egui::Context, current_instant: Instant) -> bool {
        match &self.playback_state {
            PlaybackState::Paused => false,
            PlaybackState::Playing(last_instant) => {
                if current_instant >= *last_instant + self.frame_duration {
                    self.next_frame(ctx)
                } else {
                    false
                }
            }
        }
    }

    fn next_frame(&mut self, ctx: &egui::Context) -> bool {
        let index_option = self.frame_iter.next().or_else(|| {
            self.frame_iter = Self::create_frame_iter(&self.movie);
            self.frame_iter.next()
        });

        match index_option {
            None => false,
            Some(frame_index) => {
                let palettes = SliceCache::new(self.movie.palettes());
                let tiles = SliceCache::new(self.movie.tiles());
                let movie_frame = &self.movie.frames()[frame_index];
                let frame = MovieFrame::new(ctx, &palettes, &tiles, movie_frame);
                self.current_frame = Some(frame);
                true
            }
        }
    }

    pub fn show(&mut self, ui: &mut egui::Ui) {
        if let Some(ref frame) = self.current_frame {
            let screen_size = self.movie.screen_size();
            let movie_frame_size = screen_size.to_egui() * ZOOM;
            egui::ScrollArea::both()
                .auto_shrink([false, false])
                .always_show_scroll(true)
                .show_viewport(ui, |ui, viewport| {
                    // Make sure the movie window doesn't shrink too far
                    ui.set_min_size(movie_frame_size);
                    frame.show(ui, screen_size, viewport);
                });
        }
    }

    #[inline(always)]
    fn create_frame_iter(movie: &art_extractor_core::movie::Movie) -> Range<usize> {
        0..movie.frames().len()
    }
}