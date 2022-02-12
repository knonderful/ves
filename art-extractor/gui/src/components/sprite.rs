use crate::egui;

pub struct Sprite {
    pub rect: art_extractor_core::geom_art::Rect,
    pub texture: egui::TextureHandle,
    pub hflip: bool,
    pub vflip: bool,
}
