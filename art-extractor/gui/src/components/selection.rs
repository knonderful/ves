use crate::egui;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SelectionState {
    Unselected,
    Selected,
}

impl SelectionState {
    pub fn show(&self, ui: &egui::Ui, rect: egui::Rect, line_width: f32) {
        match self {
            SelectionState::Unselected => {}
            SelectionState::Selected => {
                let width = line_width * ui.ctx().pixels_per_point();
                let rect = egui::Rect::from_min_max(
                    egui::pos2(rect.min.x - width, rect.min.y - width),
                    egui::pos2(rect.max.x + width, rect.max.y + width),
                );
                ui.painter()
                    .rect_stroke(rect, 2.0, egui::Stroke::new(width, egui::Color32::WHITE));
            }
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Selectable<T> {
    pub item: T,
    pub state: SelectionState,
}

impl<T> Selectable<T> {
    pub fn new(item: T, state: SelectionState) -> Self {
        Self { item, state }
    }
}
