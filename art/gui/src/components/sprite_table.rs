use crate::components::selection::Selectable;
use crate::components::sprite::Sprite;
use crate::egui;
use crate::egui::Sense;
use crate::ToEgui as _;

const ZOOM: f32 = 2.0;

#[derive(Clone, Copy, Debug, Default)]
#[must_use = "You should call .store()"]
struct State {
    selection_root: Option<usize>,
}

impl State {
    pub fn load(ctx: &egui::Context) -> Option<Self> {
        ctx.data().get_persisted(egui::Id::new("scroll_area"))
    }

    pub fn store(self, ctx: &egui::Context) {
        ctx.data().insert_persisted(egui::Id::new("scroll_area"), self);
    }
}

pub struct SpriteTable<'a> {
    sprites: &'a mut [Selectable<Sprite>],
    columns: usize,
}

impl<'a> SpriteTable<'a> {
    pub fn new(sprites: &'a mut [Selectable<Sprite>], columns: usize) -> Self {
        Self {
            sprites,
            columns,
        }
    }

    pub fn show(&mut self, ui: &mut egui::Ui) {
        let mut state = State::load(ui.ctx()).unwrap_or_default();

        egui::Grid::new("sprite_table")
            .spacing(egui::vec2(4.0, 4.0))
            .show(ui, |ui| {
                let from_rect = egui::Rect::from_min_size(egui::Pos2::ZERO, ui.available_size());
                let to_rect =
                    egui::Rect::from_min_size(egui::Pos2::ZERO, super::zoom_vec2(ui, ZOOM));
                let transform = egui::emath::RectTransform::from_to(from_rect, to_rect);

                let mut clicked_sprite_idx = None;
                self.sprites
                    .iter()
                    .enumerate()
                    .for_each(|(idx, selectable_sprite)| {
                        let state = &selectable_sprite.state;
                        let sprite = &selectable_sprite.item;
                        let egui_sprite_rect = sprite.rect().to_egui();

                        let rect = transform.transform_rect(egui_sprite_rect);
                        let response = ui.add(sprite.to_image(rect.size()).sense(Sense::click()));
                        if response.clicked() {
                            clicked_sprite_idx = Some(idx);
                        }
                        state.show(ui, response.rect, ZOOM);

                        if idx > 0 && (idx - 1) % self.columns == 0 {
                            ui.end_row()
                        }
                    });

                if let Some(clicked_idx) = clicked_sprite_idx {
                    let modifiers = ui.input().modifiers;
                    if modifiers.shift {
                        let range = if let Some(root) = state.selection_root {
                            if root < clicked_idx {
                                root..=clicked_idx
                            } else {
                                clicked_idx..=root
                            }
                        } else {
                            1..=0
                        };

                        if modifiers.ctrl {
                            for idx in range {
                                self.sprites[idx].state.select();
                            }
                        } else {
                            for (idx, selectable_sprite) in self.sprites.iter_mut().enumerate() {
                                selectable_sprite.state.set(range.contains(&idx));
                            }
                        }
                    } else {
                        state.selection_root = Some(clicked_idx);
                        if modifiers.ctrl {
                            self.sprites[clicked_idx].state.toggle();
                        } else {
                            for (idx, selectable_sprite) in self.sprites.iter_mut().enumerate() {
                                selectable_sprite.state.set(idx == clicked_idx);
                            }
                        }
                    }
                }
            });

        state.store(ui.ctx());
    }
}
