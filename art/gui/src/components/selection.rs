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

    pub fn toggle(&mut self) {
        match self {
            SelectionState::Unselected => *self = SelectionState::Selected,
            SelectionState::Selected => *self = SelectionState::Unselected,
        }
    }

    pub fn set(&mut self, selected: bool) {
        if selected {
            self.select();
        } else {
            self.unselect();
        }
    }

    pub fn select(&mut self) {
        *self = SelectionState::Selected;
    }

    pub fn unselect(&mut self) {
        *self = SelectionState::Unselected;
    }

    pub fn selected(&self) -> bool {
        *self == SelectionState::Selected
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

/// A range of GUI elements that have a [`SelectionState`].
#[derive(Clone, Debug, Default, serde::Serialize, serde::Deserialize)]
pub struct SelectionRange {
    selection_root: Option<usize>,
}

impl SelectionRange {
    /// Updates the selection.
    ///
    /// # Arguments
    ///
    /// * `ui`: The [`Ui`](egui::Ui).
    /// * `clicked_idx`: The index of the item that was clicked.
    /// * `values`: A slice of all available items.
    /// * `map_fn`: A mapping function from `T` to [`SelectionState`].
    pub fn update<T>(
        &mut self,
        ui: &egui::Ui,
        clicked_idx: usize,
        values: &mut [T],
        map_fn: impl Fn(&mut T) -> &mut SelectionState,
    ) {
        let modifiers = ui.input().modifiers;
        if modifiers.shift {
            let range = if let Some(root) = self.selection_root {
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
                    let map_fn1 = map_fn(&mut values[idx]);
                    map_fn1.select();
                }
            } else {
                for (idx, selectable_sprite) in values.iter_mut().enumerate() {
                    map_fn(selectable_sprite).set(range.contains(&idx));
                }
            }
        } else {
            self.selection_root = Some(clicked_idx);
            if modifiers.ctrl {
                map_fn(&mut values[clicked_idx]).toggle();
            } else {
                for (idx, selectable_sprite) in values.iter_mut().enumerate() {
                    map_fn(selectable_sprite).set(idx == clicked_idx);
                }
            }
        }
    }
}
