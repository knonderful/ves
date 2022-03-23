use crate::components::selection::{Selectable, SelectionRange};
use crate::components::sprite::Sprite;
use crate::egui;
use crate::egui::Sense;
use crate::ToEgui as _;

const ZOOM: f32 = 2.0;

#[derive(Clone, Debug, Default)]
#[must_use = "You should call .store()"]
struct State {
    selection: SelectionRange,
}

impl State {
    pub fn load(ctx: &egui::Context) -> Option<Self> {
        ctx.data().get_persisted(egui::Id::new("sprite_table"))
    }

    pub fn store(self, ctx: &egui::Context) {
        ctx.data()
            .insert_persisted(egui::Id::new("sprite_table"), self);
    }
}

pub struct SpriteTable<'a> {
    sprites: &'a mut [Selectable<Sprite>],
    columns: usize,
}

impl<'a> SpriteTable<'a> {
    pub fn new(sprites: &'a mut [Selectable<Sprite>], columns: usize) -> Self {
        Self { sprites, columns }
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
                    state
                        .selection
                        .update(ui, clicked_idx, self.sprites, |sprite| &mut sprite.state);
                }
            });

        state.store(ui.ctx());
    }
}
