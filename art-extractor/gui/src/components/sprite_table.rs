use crate::components::movie::{correct_uv, DEFAULT_UV, zoom_vec2};
use crate::components::sprite::Sprite;
use crate::egui;
use crate::ToEgui as _;

pub struct SpriteTable<'a> {
    sprites: &'a [Sprite],
    columns: usize,
}

impl<'a> SpriteTable<'a> {
    pub fn new(sprites: &'a [Sprite], columns: usize) -> Self {
        Self { sprites, columns }
    }

    pub fn show(&mut self, ui: &mut egui::Ui) {
        egui::Grid::new("sprite_table")
            .spacing(egui::vec2(2.0, 2.0))
            .show(ui, |ui| {
                let from_rect = egui::Rect::from_min_size(egui::Pos2::ZERO, ui.available_size());
                let to_rect = egui::Rect::from_min_size(egui::Pos2::ZERO, zoom_vec2(ui));
                let transform = egui::emath::RectTransform::from_to(from_rect, to_rect);

                self.sprites.iter().enumerate().for_each(|(idx, sprite)| {
                    let egui_sprite_rect = sprite.rect.to_egui();

                    let rect = transform.transform_rect(egui_sprite_rect);
                    let image = egui::Image::new(&sprite.texture, rect.size()).uv(correct_uv(
                        DEFAULT_UV,
                        sprite.hflip,
                        sprite.vflip,
                    ));
                    ui.add(image);

                    if idx > 0 && idx % self.columns == 0 {
                        ui.end_row()
                    }
                });
            });
    }
}
