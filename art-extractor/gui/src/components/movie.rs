use super::sprite::Sprite;
use crate::components::cursor::Cursor;
use crate::components::mouse::MouseInteractionTracker;
use crate::components::selection::{Selectable, SelectionState};
use crate::egui;
use crate::egui::ImageData;
use crate::ToEgui as _;
use std::time::{Duration, Instant};
use ves_cache::SliceCache;
use ves_geom::RectIntersection;

struct MovieFrame<'a> {
    sprites: &'a [Selectable<Sprite>],
}

const ZOOM: f32 = 2.0;

impl<'a> MovieFrame<'a> {
    /// Creates a new instance.
    pub fn new(sprites: &'a [Selectable<Sprite>]) -> Self {
        Self { sprites }
    }

    pub fn show(
        &self,
        ui: &mut egui::Ui,
        screen_size: art_extractor_core::geom_art::Size,
        viewport: egui::Rect,
    ) {
        // TODO: It seems like the UI adds spacing of an extra 8px when an image is exactly on the edge, causing the scrollbars to resize
        //       when a sprite wraps around.

        // Use a from and to rectangle to transform (translate and scale) the frame in the movie window.
        let from_rect = egui::Rect::from_min_size(egui::Pos2::ZERO, ui.available_size());
        // NOTE: The window manager may have altered the pixels per point (e.g. to scale up the entire UI with the selected font size.
        //       Unfortunately, this is not very good for pixel-perfect rendering. We could set the pixels_per_point to 1 for the entire
        //       application, which would work, but then the UI of the application would not scale with what the user is "used to". Instead,
        //       here we correct our calculations by dividing by pixels_per_point.
        let to_rect = egui::Rect::from_min_size(
            ui.clip_rect().min + egui::vec2(-viewport.left(), -viewport.top()),
            super::zoom_vec2(ui, ZOOM),
        );
        let transform = egui::emath::RectTransform::from_to(from_rect, to_rect);

        let intersect_pos = screen_size.as_rect().max;

        // Collect all selection states with their rects and render them after all the images have
        // been added, since we want the selection boxes to appear over all sprites.
        let mut states_with_rect = Vec::with_capacity(self.sprites.len());

        self.sprites.iter().rev().for_each(|selectable_sprite| {
            let state = &selectable_sprite.state;
            let sprite = &selectable_sprite.item;
            match sprite.rect.intersect_point(intersect_pos) {
                // No intersections; this means the sprite fits entirely on the screen
                RectIntersection::None => {
                    let rect = transform.transform_rect(sprite.rect.to_egui());
                    ui.put(rect, sprite.to_image(rect.size()));
                    states_with_rect.push((state, rect));
                }
                // Treat all other cases generically
                intersection => {
                    intersection.for_each(|rect| {
                        let egui_dest_rect = art_extractor_core::geom_art::Rect::new_from_size(
                            (
                                rect.min_x() % screen_size.width,
                                rect.min_y() % screen_size.height,
                            ),
                            rect.size(),
                        )
                        .to_egui();

                        let dest_rect = transform.transform_rect(egui_dest_rect);
                        let image = egui::Image::new(&sprite.texture, dest_rect.size())
                            .uv(sprite.partial_uv(rect));

                        ui.put(dest_rect, image);
                        states_with_rect.push((state, dest_rect));
                    });
                }
            }
        });

        for state in states_with_rect {
            let (state, rect) = state;
            state.show(ui, rect, ZOOM);
        }
    }
}

#[derive(Clone, Debug)]
enum PlaybackState {
    /// The "paused" state.
    Paused,
    /// The "playing" state. The argument is the instant that the last frame was set.
    Playing(Instant),
}

pub struct Movie {
    movie: art_extractor_core::movie::Movie,
    frame_cursor: Cursor,
    frame_duration: Duration,
    playback_state: PlaybackState,
    playback_repeat: bool,
    /// The current frame. The first item is the current frame number. The second are the sprites in
    /// the frame.
    current_frame: Option<(usize, Vec<Selectable<Sprite>>)>,
    control_messages: Vec<MovieControlMessage>,
    mouse_tracker: MouseInteractionTracker,
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
            control_messages: Vec::with_capacity(16),
            mouse_tracker: Default::default(),
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
        while let Some(msg) = self.control_messages.pop() {
            self.handle_control_message(msg, current_instant);
        }

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

        let mut sprites = if let Some((_, mut sprites)) = self.current_frame.take() {
            sprites.clear();
            sprites
        } else {
            Vec::with_capacity(movie_frame.sprites().len())
        };

        for (i, sprite) in movie_frame.sprites().iter().enumerate() {
            let gui_sprite = Sprite::create(sprite, &palettes, &tiles, |color_image| {
                ctx.load_texture("something", ImageData::Color(color_image))
            });
            let selection_state = if i % 2 == 0 {
                SelectionState::Selected
            } else {
                SelectionState::Unselected
            };
            sprites.push(Selectable::new(gui_sprite, selection_state));
        }

        self.current_frame = Some((pos, sprites));

        true
    }

    fn handle_control_message(&mut self, msg: MovieControlMessage, current_instant: Instant) {
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
            MovieControlMessage::Jump(msg) => match msg {
                JumpMessage::Start => self.frame_cursor.reset(),
                JumpMessage::End => {
                    self.frame_cursor.move_forward(usize::MAX);
                }
            },
            MovieControlMessage::SetRepeat(val) => {
                self.playback_repeat = val;
            }
        }
    }

    pub fn show(&mut self, ui: &mut egui::Ui) {
        ui.with_layout(egui::Layout::bottom_up(egui::Align::Center), |ui| {
            MovieControls::new(self.playback_state.clone(), self.playback_repeat, |msg| {
                self.control_messages.push(msg)
            })
            .show(ui);

            // Some space between controls and render window
            ui.add_space(8.0);

            if let Some((_, frame)) = &self.current_frame {
                let screen_size = self.movie.screen_size();
                let movie_frame_size = screen_size.to_egui() * ZOOM;

                // Set a reasonable minimal size. This also results in good defaults (currently).
                let scrollbar_width = ui.style().spacing.scroll_bar_width;
                ui.allocate_ui(
                    egui::vec2(256.0, 224.0) * ZOOM + egui::vec2(scrollbar_width, scrollbar_width),
                    |ui| {
                        egui::ScrollArea::both()
                            .auto_shrink([false, false])
                            .always_show_scroll(true)
                            .show_viewport(ui, |ui, viewport| {
                                // Make sure the movie canvas doesn't shrink too far
                                ui.set_min_size(movie_frame_size);

                                MovieFrame::new(frame).show(ui, screen_size, viewport);

                                // This also "steals" the interaction of the parent, which in this
                                // case causes the ScrollArea not to scroll on drag (which is what
                                // we want).
                                let response = ui.interact(
                                    ui.min_rect(),
                                    ui.id(),
                                    egui::Sense::click_and_drag(),
                                );

                                use crate::components::mouse::{DragEvent, MouseInteraction};

                                if let Some(event) = self.mouse_tracker.update(&response) {
                                    match event {
                                        MouseInteraction::Click(_) => { /* do nothing for now */ }
                                        MouseInteraction::Drag(event) => match event {
                                            DragEvent::Start(_) => {}
                                            DragEvent::Update(rect) => {
                                                ui.painter().rect_stroke(
                                                    rect,
                                                    0.0,
                                                    egui::Stroke::new(
                                                        ui.ctx().pixels_per_point(),
                                                        egui::Color32::from_rgb(255, 255, 255),
                                                    ),
                                                );
                                            }
                                            DragEvent::Finished(_) => {}
                                        },
                                    }
                                }
                            });
                    },
                );

                // HACK: This seems to be necessary to prevent the scroll area from rendering into
                //       the window header.
                ui.add_space(8.0);
            } else {
                ui.label("No movie frame available.");
            }
        });
    }

    pub fn sprites(&self) -> Option<&[Selectable<Sprite>]> {
        if let Some((_, sprites)) = &self.current_frame {
            Some(sprites.as_slice())
        } else {
            None
        }
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
        Self {
            playback_state,
            playback_repeat,
            sink,
        }
    }
}

impl<Sink> MovieControls<Sink>
where
    Sink: FnMut(MovieControlMessage),
{
    fn add_button(
        &mut self,
        ui: &mut egui::Ui,
        icon: &'static str,
        on_click_fn: impl FnOnce(&mut Sink),
    ) {
        if ui.button(icon).clicked() {
            on_click_fn(&mut self.sink);
        }
    }

    fn add_button_simple(
        &mut self,
        ui: &mut egui::Ui,
        icon: &'static str,
        message: MovieControlMessage,
    ) {
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
            self.add_button_simple(
                ui,
                "🔁",
                MovieControlMessage::SetRepeat(!self.playback_repeat),
            );
        });
    }
}
