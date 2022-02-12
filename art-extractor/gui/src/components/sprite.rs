use crate::egui;

pub const DEFAULT_UV: egui::Rect =
    egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0));

fn correct_uv(rect: egui::Rect, hflip: bool, vflip: bool) -> egui::Rect {
    if hflip {
        if vflip {
            egui::Rect::from_min_max(rect.max, rect.min)
        } else {
            egui::Rect::from_min_max(
                egui::pos2(rect.max.x, rect.min.y),
                egui::pos2(rect.min.x, rect.max.y),
            )
        }
    } else {
        if vflip {
            egui::Rect::from_min_max(
                egui::pos2(rect.min.x, rect.max.y),
                egui::pos2(rect.max.x, rect.min.y),
            )
        } else {
            rect
        }
    }
}

pub struct Sprite {
    pub rect: art_extractor_core::geom_art::Rect,
    pub texture: egui::TextureHandle,
    pub hflip: bool,
    pub vflip: bool,
}

impl Sprite {
    pub fn to_image(&self, size: egui::Vec2) -> egui::Image {
        egui::Image::new(&self.texture, size).uv(correct_uv(
            DEFAULT_UV,
            self.hflip,
            self.vflip,
        ))
    }
}
