mod components;

use crate::components::movie::Movie;
use crate::components::selection::SelectionState;
use crate::components::sprite_table::SpriteTable;
use eframe::{egui, epi};
use std::time::Instant;
use log::info;
use ves_art_core::geom_art::ArtworkSpaceUnit;
use crate::components::sprite_details::SpriteDetails;
use crate::components::window::Window;

#[derive(Default)]
struct ArtDirectorApp {
    movie: Option<Movie>,
}

impl epi::App for ArtDirectorApp {
    fn update(&mut self, ctx: &egui::Context, frame: &epi::Frame) {
        let current_instant = Instant::now();

        if let Some(ref mut movie) = self.movie {
            if movie.update(ctx, current_instant) {
                ctx.request_repaint();
            }
        }

        // Auto-load hack
        if self.movie.is_none() {
            let mut input_file = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
            input_file.push("../../yoshi_run.bincode");
            let file = std::fs::File::open(input_file).unwrap();
            match bincode::deserialize_from::<_, ves_art_core::movie::Movie>(file) {
                Ok(core_movie) => {
                    let gui_movie = Movie::new(core_movie);
                    // gui_movie.play(current_instant);
                    self.movie = Some(gui_movie);
                    info!("Successfully loaded test movie.");
                }
                Err(err) => {
                    info!("Could not load test movie: {}", err);
                }
            }
        }

        egui::TopBottomPanel::top("main_menu").show(ctx, |ui| {
            ui.horizontal(|ui| {
                // Mini menu icons
                ui.with_layout(egui::Layout::right_to_left(), |ui| {
                    egui::global_dark_light_mode_switch(ui);
                });
            })
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            Window::new("Movie").show(ui.ctx(), |ui| match &mut self.movie {
                None => {
                    ui.label("No movie loaded.");
                }
                Some(movie) => {
                    movie.show(ui);
                }
            });

            Window::new("Sprites").show(ui.ctx(), |ui| {
                match self.movie.as_mut().and_then(|movie| movie.sprites_mut()) {
                    None => {
                        ui.label("No movie loaded.");
                    }
                    Some(sprites) => {
                        SpriteTable::new(sprites, 8).show(ui);
                    }
                }
            });

            Window::new("Sprite Details").show(ui.ctx(), |ui| {
                match self.movie.as_ref().and_then(|movie| movie.sprites()) {
                    None => {
                        ui.label("No movie loaded.");
                    }
                    Some(sprites) => {
                        let selected_sprites: Vec<_> = sprites
                            .iter()
                            .enumerate()
                            .filter(|(_, s)| s.state == SelectionState::Selected)
                            .collect();
                        match selected_sprites.len() {
                            0 => {
                                ui.label("No sprite selected.");
                            }
                            1 => {
                                let (index, sprite) = selected_sprites[0];
                                SpriteDetails::new(index, &sprite.item).show(ui);
                            }
                            _ => {
                                ui.label("Multiple sprites selected.");
                            }
                        };
                    }
                }
            });
        });

        // Resize the native window to be just the size we need it to be:
        frame.set_window_size(ctx.used_size());
    }

    fn name(&self) -> &str {
        "VES Art Director"
    }
}

trait IntoF32 {
    fn into_f32(self) -> f32;
}

impl IntoF32 for u32 {
    #[inline(always)]
    fn into_f32(self) -> f32 {
        u16::try_from(self).unwrap().into()
    }
}

impl IntoF32 for ArtworkSpaceUnit {
    #[inline(always)]
    fn into_f32(self) -> f32 {
        self.raw().into_f32()
    }
}

/// Trait for converting types into their "egui" counterparts.
trait ToEgui {
    type Out;

    /// Converts the type.
    fn to_egui(&self) -> Self::Out;
}

impl ToEgui for ves_art_core::geom_art::Rect {
    type Out = egui::Rect;

    #[inline(always)]
    fn to_egui(&self) -> Self::Out {
        // We have to convert from an inclusive (integer-based) to an exclusive (float-based) space, hence the +1
        egui::Rect::from_min_max(
            egui::pos2(self.min_x().into_f32(), self.min_y().into_f32()),
            egui::pos2(self.max_x().into_f32() + 1.0, self.max_y().into_f32() + 1.0),
        )
    }
}

impl ToEgui for ves_art_core::geom_art::Size {
    type Out = egui::Vec2;

    #[inline(always)]
    fn to_egui(&self) -> Self::Out {
        egui::Vec2::new(self.width.into_f32(), self.height.into_f32())
    }
}

fn main() {
    simple_logger::SimpleLogger::new().init().unwrap();

    let options = eframe::NativeOptions::default();
    eframe::run_native(Box::new(ArtDirectorApp::default()), options);
}
