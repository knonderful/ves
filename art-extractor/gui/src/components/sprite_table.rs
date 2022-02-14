use crate::components::selection::Selectable;
use crate::components::sprite::Sprite;
use crate::egui;
use crate::ToEgui as _;

const ZOOM: f32 = 2.0;

pub struct SpriteTable<'a> {
    sprites: &'a [Selectable<Sprite>],
    columns: usize,
}

impl<'a> SpriteTable<'a> {
    pub fn new(sprites: &'a [Selectable<Sprite>], columns: usize) -> Self {
        Self { sprites, columns }
    }

    pub fn show(&mut self, ui: &mut egui::Ui) {
        egui::Grid::new("sprite_table")
            .spacing(egui::vec2(2.0, 2.0))
            .show(ui, |ui| {
                let from_rect = egui::Rect::from_min_size(egui::Pos2::ZERO, ui.available_size());
                let to_rect =
                    egui::Rect::from_min_size(egui::Pos2::ZERO, super::zoom_vec2(ui, ZOOM));
                let transform = egui::emath::RectTransform::from_to(from_rect, to_rect);

                self.sprites
                    .iter()
                    .enumerate()
                    .for_each(|(idx, selectable_sprite)| {
                        let state = &selectable_sprite.state;
                        let sprite = &selectable_sprite.item;
                        let egui_sprite_rect = sprite.rect.to_egui();

                        let rect = transform.transform_rect(egui_sprite_rect);
                        let response = ui.add(sprite.to_image(rect.size()));
                        state.show(ui, response.rect, ZOOM);

                        if idx > 0 && idx % self.columns == 0 {
                            ui.end_row()
                        }
                    });
            });
    }
}
