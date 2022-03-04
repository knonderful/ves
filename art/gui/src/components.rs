pub mod cursor;
pub mod mouse;
pub mod movie;
pub mod selection;
pub mod sprite;
pub mod sprite_table;

use crate::egui;

/// Returns a "zoom" [`egui::Vec2`] for use in a pixel-perfect [`egui::emath::RectTransform`].
///
/// # Arguments
///
/// * `ui`: The UI.
/// * `factor`: The zoom factor.
///
/// returns: The [`egui::Vec2`].
pub fn zoom_vec2(ui: &egui::Ui, factor: f32) -> egui::Vec2 {
    (factor / ui.ctx().pixels_per_point()) * ui.available_size()
}
