use crate::components::sprite::Sprite;
use crate::egui;
use crate::ToEgui as _;

const ZOOM: f32 = 2.0;

pub struct SpriteDetails<'a> {
    index: usize,
    sprite: &'a Sprite,
}

impl<'a> SpriteDetails<'a> {
    pub fn new(index: usize, sprite: &'a Sprite) -> Self {
        Self {  index, sprite }
    }

    pub fn show(&mut self, ui: &mut egui::Ui) {
        ui.vertical(|ui| {
            let from_rect = egui::Rect::from_min_size(egui::Pos2::ZERO, ui.available_size());
            let to_rect = egui::Rect::from_min_size(egui::Pos2::ZERO, super::zoom_vec2(ui, ZOOM));
            let transform = egui::emath::RectTransform::from_to(from_rect, to_rect);
            let sprite = self.sprite;
            let egui_sprite_rect = sprite.rect().to_egui();
            let rect = transform.transform_rect(egui_sprite_rect);

            ui.add(sprite.to_image(rect.size()));
            ui.end_row();
            egui::Grid::new("sprite_table")
                .spacing(egui::vec2(10.0, 5.0))
                .show(ui, |ui| {
                    ui.label("Index");
                    ui.label(format!("{}", self.index));
                    ui.end_row();
                    ui.label("Tile");
                    ui.label(format!("{}", sprite.sprite().tile().value()));
                    ui.end_row();
                    ui.label("Palette");
                    ui.label(format!("{}", sprite.sprite().palette().value()));
                    ui.end_row();
                    ui.label("Position");
                    ui.label(format!("{:?}", sprite.sprite().position()));
                    ui.end_row();
                    ui.label("H-flip");
                    ui.label(format!("{:?}", sprite.sprite().h_flip()));
                    ui.end_row();
                    ui.label("V-flip");
                    ui.label(format!("{:?}", sprite.sprite().v_flip()));
                    ui.end_row();
                });

        });
    }
}
