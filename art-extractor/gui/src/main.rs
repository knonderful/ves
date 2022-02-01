mod movie;

use crate::movie::{Movie, SpriteTable};
use art_extractor_core::geom_art::ArtworkSpaceUnit;
use chrono::{DateTime, Local};
use eframe::{egui, epi};
use std::collections::VecDeque;
use std::time::Instant;

#[derive(Debug, Eq, PartialEq, Clone)]
enum MainMode {
    Movie,
}

struct LogEntry {
    timestamp: DateTime<Local>,
    message: String,
}

struct ArtDirectorApp {
    main_mode: MainMode,
    show_log: bool,
    log: VecDeque<LogEntry>,
    movie: Option<Movie>,
}

impl Default for ArtDirectorApp {
    fn default() -> Self {
        Self {
            main_mode: MainMode::Movie,
            show_log: false,
            log: Default::default(),
            movie: None,
        }
    }
}

impl ArtDirectorApp {
    #[allow(unused)]
    fn log(&mut self, msg: impl AsRef<str>) {
        const MAX_LOG_ENTRIES: usize = 50;
        let log = &mut self.log;
        if log.len() >= MAX_LOG_ENTRIES {
            log.pop_front();
        }

        log.push_back(LogEntry {
            timestamp: chrono::Local::now(),
            message: String::from(msg.as_ref()),
        });
    }
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
            input_file.push("../../test_movie.bincode");
            let file = std::fs::File::open(input_file).unwrap();
            match bincode::deserialize_from::<_, art_extractor_core::movie::Movie>(file) {
                Ok(core_movie) => {
                    let mut gui_movie = Movie::new(core_movie);
                    gui_movie.play(current_instant);
                    self.movie = Some(gui_movie);
                    self.log("Successfully loaded test movie.");
                }
                Err(err) => {
                    self.log(format!("Could not load test movie: {}", err));
                }
            }
        }

        egui::TopBottomPanel::top("main_menu").show(ctx, |ui| {
            ui.horizontal(|ui| {
                // Mode selection items
                ui.with_layout(egui::Layout::left_to_right(), |ui| {
                    ui.selectable_value(&mut self.main_mode, MainMode::Movie, "Movie");
                });
                // Mini menu icons
                ui.with_layout(egui::Layout::right_to_left(), |ui| {
                    egui::global_dark_light_mode_switch(ui);

                    let log_toggle = ui
                        .add(egui::Button::new("📋").frame(false))
                        .on_hover_text("Toggle application log");
                    if log_toggle.clicked() {
                        self.show_log = !self.show_log;
                    }
                });
            })
        });

        if self.show_log {
            egui::TopBottomPanel::bottom("application_log")
                .height_range(100.0..=100.0)
                .show(ctx, |ui| {
                    egui::ScrollArea::both().stick_to_bottom().show(ui, |ui| {
                        egui::Grid::new("log_grid")
                            .striped(true)
                            .max_col_width(f32::INFINITY)
                            .show(ui, |ui| {
                                for entry in self.log.iter() {
                                    ui.label(entry.timestamp.format("%H:%M:%S").to_string());
                                    ui.label(entry.message.as_str());
                                    ui.end_row();
                                }
                            });
                    });

                    // egui::ScrollArea::vertical().stick_to_bottom().show(ui, |ui| {
                    //     // The '&mut log.as_str()' makes it a read-only TextBuffer
                    //     ui.add(egui::TextEdit::multiline(&mut log.as_str())
                    //         .text_style(LOG_FONT)
                    //         .desired_width(f32::INFINITY));
                    // });
                });
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            egui::Window::new("Movie").show(ui.ctx(), |ui| match self.main_mode {
                MainMode::Movie => match &mut self.movie {
                    None => {
                        ui.label("No movie loaded.");
                    }
                    Some(movie) => {
                        movie.show(ui);
                    }
                },
            });

            egui::Window::new("Sprites").show(ui.ctx(), |ui| {
                match self.movie.as_ref().and_then(|movie| movie.sprites()) {
                    None => {
                        ui.label("No movie loaded.");
                    }
                    Some(sprites) => {
                        SpriteTable::new(sprites, 8).show(ui);
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

impl ToEgui for art_extractor_core::geom_art::Rect {
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

impl ToEgui for art_extractor_core::geom_art::Size {
    type Out = egui::Vec2;

    #[inline(always)]
    fn to_egui(&self) -> Self::Out {
        egui::Vec2::new(self.width.into_f32(), self.height.into_f32())
    }
}

fn main() {
    let options = eframe::NativeOptions::default();
    eframe::run_native(Box::new(ArtDirectorApp::default()), options);
}
