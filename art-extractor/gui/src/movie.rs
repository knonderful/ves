use std::ops::Index;
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
    hflip: bool,
    vflip: bool,
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
                false, false, // We do flipping in the mesh/Image instead of in the texture (using UV)
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
                hflip: sprite.h_flip(),
                vflip: sprite.v_flip(),
            };

            sprites.push(gui_sprite);
        }

        Self { sprites }
    }

    fn correct_uv(rect: egui::Rect, hflip: bool, vflip: bool) -> egui::Rect {
        if hflip {
            if vflip {
                egui::Rect::from_min_max(rect.max, rect.min)
            } else {
                egui::Rect::from_min_max(egui::pos2(rect.max.x, rect.min.y), egui::pos2(rect.min.x, rect.max.y))
            }
        } else {
            if vflip {
                egui::Rect::from_min_max(egui::pos2(rect.min.x, rect.max.y), egui::pos2(rect.max.x, rect.min.y))
            } else {
                rect
            }
        }
    }

    pub fn show(&self, ui: &mut egui::Ui, screen_size: art_extractor_core::geom_art::Size, viewport: egui::Rect) {
        // TODO: The scaling is not pixel-perfect by default. This has to do with the texture filtering in the rendering component.
        //       Currently this requires a hack in egui_glow, since there is no way for the application code to control this.
        // TODO: It seems like the UI adds spacing of an extra 8px when an image is exactly on the edge, causing the scrollbars to resize
        //       when a sprite wraps around.

        // Use a from and to rectangle to transform (translate and scale) the frame in the movie window.
        let from_rect = egui::Rect::from_min_size(egui::Pos2::ZERO, ui.available_size());
        // NOTE: The window manager may have altered the pixels per point (e.g. to scale up the entire UI with the selected font size.
        //       Unfortunately, this is not very good for pixel-perfect rendering. We could set the pixels_per_point to 1 for the entire
        //       application, which would work, but then the UI of the application would not scale with what the user is "used to". Instead,
        //       here we correct our calculations by dividing by pixels_per_point.
        let to_rect = egui::Rect::from_min_size(egui::pos2(-viewport.left(), -viewport.top()), (ZOOM / ui.ctx().pixels_per_point()) * ui.available_size());
        let transform = egui::emath::RectTransform::from_to(from_rect, to_rect);

        let intersect_pos = screen_size.as_rect().max;

        const DEFAULT_UV: egui::Rect = egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0));

        self.sprites.iter().for_each(|sprite| {
            let egui_sprite_rect = sprite.rect.to_egui();
            match sprite.rect.intersect_point(intersect_pos) {
                // No intersections; this means the sprite fits entirely on the screen
                RectIntersection::None => {
                    let rect = transform.transform_rect(egui_sprite_rect);
                    let image = egui::Image::new(&sprite.texture, rect.size())
                        .uv(Self::correct_uv(DEFAULT_UV, sprite.hflip, sprite.vflip));

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
                        let uv = egui::Rect::from_min_max(egui::pos2(u_x, u_y), egui::pos2(v_x, v_y));
                        let image = egui::Image::new(&sprite.texture, dest_rect.size())
                            .uv(Self::correct_uv(uv, sprite.hflip, sprite.vflip));

                        ui.put(dest_rect, image);
                    });
                }
            }
        });
    }
}

#[derive(Clone, Debug)]
enum PlaybackState {
    /// The "paused" state.
    Paused,
    /// The "playing" state. The argument is the instant that the last frame was set.
    Playing(Instant),
}

/// A cursor represents a position in a range or slice.
///
/// The cursor can be moved forward and backward, but can never exceed the bounds of the range.
struct Cursor {
    length: usize,
    position: usize,
}

impl Cursor {
    /// Creates a new instance.
    ///
    /// # Arguments
    ///
    /// * `length`: The number of positions.
    pub fn new(length: usize) -> Self {
        assert_ne!(length, 0);
        Self {
            length,
            position: 0,
        }
    }

    /// Retrieves the current position.
    pub fn position(&self) -> usize {
        self.position
    }

    /// Resets the cursor to the initial position.
    pub fn reset(&mut self) {
        self.position = 0;
    }

    /// Moves the cursor backward at most the provided number of steps.
    ///
    /// # Parameters
    /// * `count`: The maximum number of steps to move the cursor.
    ///
    /// # Returns
    /// The number of steps that the cursor was actually moved.
    #[allow(unused)]
    pub fn move_backward(&mut self, count: usize) -> usize {
        let distance = self.position.min(count);
        self.position -= distance;
        distance
    }

    /// Moves the cursor forward at most the provided number of steps.
    ///
    /// # Parameters
    /// * `count`: The maximum number of steps to move the cursor.
    ///
    /// # Returns
    /// The number of steps that the cursor was actually moved.
    pub fn move_forward(&mut self, count: usize) -> usize {
        let distance = ((self.length - 1) - self.position).min(count);
        self.position += distance;
        distance
    }

    /// Moves the cursor forward one step.
    ///
    /// # Returns
    /// The new position of the cursor or `None` if the cursor is at the upper bound.
    pub fn next(&mut self) -> Option<usize> {
        if self.move_forward(1) == 0 {
            None
        } else {
            Some(self.position)
        }
    }
}

pub struct Movie {
    movie: art_extractor_core::movie::Movie,
    frame_cursor: Cursor,
    frame_duration: Duration,
    playback_state: PlaybackState,
    playback_repeat: bool,
    current_frame: Option<(usize, MovieFrame)>,
}

impl Movie {
    /// Creates a new instance.
    ///
    /// # Arguments
    ///
    /// * `movie`: The movie.
    pub fn new(movie: art_extractor_core::movie::Movie) -> Self {
        let frame_cursor = Cursor::new(movie.frames().len());
        let frame_duration = Duration::from_secs(1) / movie.frame_rate().fps();
        Self {
            movie,
            frame_cursor,
            frame_duration,
            playback_state: PlaybackState::Paused,
            playback_repeat: false,
            current_frame: None,
        }
    }

    pub fn play(&mut self, current_instant: Instant) {
        match self.playback_state {
            PlaybackState::Paused => {
                self.playback_state = PlaybackState::Playing(current_instant);
            }
            PlaybackState::Playing(_) => {} // do nothing
        }
    }

    pub fn pause(&mut self) {
        self.playback_state = PlaybackState::Paused;
    }

    pub fn update(&mut self, ctx: &egui::Context, current_instant: Instant) -> bool {
        // TODO: Process enqueued MovieControlMessages here, instead of directly in the show() call.

        match &self.playback_state {
            PlaybackState::Paused => {}
            PlaybackState::Playing(last_frame_instant) => {
                let mut delta = current_instant - *last_frame_instant;
                let frame_duration = self.frame_duration;
                // Skip frames until we've exhausted the delta
                while delta >= frame_duration {
                    if self.frame_cursor.next().is_none() {
                        if !self.playback_repeat {
                            self.pause();
                            return false;
                        }
                        self.frame_cursor.reset();
                    }
                    delta -= frame_duration;
                }
                self.playback_state = PlaybackState::Playing(current_instant - delta);
            }
        }

        self.render_frame(ctx)
        // true
    }

    fn render_frame(&mut self, ctx: &egui::Context) -> bool {
        let pos = self.frame_cursor.position();
        // Only render the frame if the position has changed
        if let Some((last_pos, _)) = &self.current_frame {
            if pos == *last_pos {
                return false;
            }
        }

        let palettes = SliceCache::new(self.movie.palettes());
        let tiles = SliceCache::new(self.movie.tiles());
        let movie_frame = &self.movie.frames()[pos];
        let frame = MovieFrame::new(ctx, &palettes, &tiles, movie_frame);
        self.current_frame = Some((pos, frame));
        true
    }

    pub fn show(&mut self, ui: &mut egui::Ui, current_instant: Instant) {
        // TODO: It is probably better to enqueue the messages somewhere and not process them until update(). Especially since external
        //       entities might want to enqueue messages (like global hotkey handlers).
        egui::TopBottomPanel::bottom("movie_controls").show(ui.ctx(), |ui| {
            MovieControls::new(self.playback_state.clone(), self.playback_repeat, |msg: MovieControlMessage| {
                match msg {
                    MovieControlMessage::Play => {
                        self.play(current_instant);
                    }
                    MovieControlMessage::Pause => {
                        self.pause();
                    }
                    MovieControlMessage::SkipBackward(count) => {
                        self.frame_cursor.move_backward(count);
                    }
                    MovieControlMessage::SkipForward(count) => {
                        self.frame_cursor.move_forward(count);
                    }
                    MovieControlMessage::Jump(msg) => {
                        match msg {
                            JumpMessage::Start => self.frame_cursor.reset(),
                            JumpMessage::End => { self.frame_cursor.move_forward(usize::MAX); }
                        }
                    }
                    MovieControlMessage::SetRepeat(val) => {
                        self.playback_repeat = val;
                    }
                }
            }).show(ui);
        });

        egui::CentralPanel::default().show(ui.ctx(), |ui| {
            if let Some((_, frame)) = &self.current_frame {
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
        });
    }
}

#[derive(Clone, Debug)]
enum JumpMessage {
    Start,
    End,
}

#[derive(Clone, Debug)]
enum MovieControlMessage {
    Play,
    Pause,
    SkipBackward(usize),
    SkipForward(usize),
    Jump(JumpMessage),
    SetRepeat(bool),
}

struct MovieControls<Sink> {
    playback_state: PlaybackState,
    playback_repeat: bool,
    sink: Sink,
}

impl<Sink> MovieControls<Sink> {
    fn new(playback_state: PlaybackState, playback_repeat: bool, sink: Sink) -> Self {
        Self { playback_state, playback_repeat, sink }
    }
}

impl<Sink> MovieControls<Sink> where
    Sink: FnMut(MovieControlMessage),
{
    fn add_button(&mut self, ui: &mut egui::Ui, icon: &'static str, on_click_fn: impl FnOnce(&mut Sink)) {
        if ui.button(icon).clicked() {
            on_click_fn(&mut self.sink);
        }
    }

    fn add_button_simple(&mut self, ui: &mut egui::Ui, icon: &'static str, message: MovieControlMessage) {
        self.add_button(ui, icon, |sink| sink(message));
    }

    fn show(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            self.add_button_simple(ui, "⏮", MovieControlMessage::Jump(JumpMessage::Start));
            self.add_button(ui, "<", |sink| {
                sink(MovieControlMessage::Pause);
                sink(MovieControlMessage::SkipBackward(1));
            });
            if let PlaybackState::Playing(_) = self.playback_state {
                self.add_button_simple(ui, "⏸", MovieControlMessage::Pause);
            } else {
                self.add_button_simple(ui, "▶", MovieControlMessage::Play);
            }
            self.add_button(ui, "⏹", |sink| {
                sink(MovieControlMessage::Pause);
                sink(MovieControlMessage::Jump(JumpMessage::Start));
            });
            self.add_button(ui, ">", |sink| {
                sink(MovieControlMessage::Pause);
                sink(MovieControlMessage::SkipForward(1));
            });
            self.add_button_simple(ui, "⏭", MovieControlMessage::Jump(JumpMessage::End));
            self.add_button_simple(ui, "🔁", MovieControlMessage::SetRepeat(!self.playback_repeat));
        });
    }
}