use crate::egui;

#[derive(Clone, Debug, Default)]
pub struct MouseInteractionTracker {
    drag_state: DragState,
}

impl MouseInteractionTracker {
    pub fn update(&mut self, response: &egui::Response) -> Option<MouseInteraction> {
        if response.clicked() {
            self.drag_state.reset();
            // Since the response was clicked, this *must* be Some, hence the unwrap.
            let pos = response.interact_pointer_pos().unwrap();
            Some(MouseInteraction::Click(pos))
        } else {
            self.drag_state.update(response).map(MouseInteraction::Drag)
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum MouseInteraction {
    Click(egui::Pos2),
    Drag(DragEvent),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DragEvent {
    Start(egui::Pos2),
    Update(egui::Rect),
    Finished(egui::Rect),
}

#[derive(Clone, Debug, Default)]
struct DragState {
    /// The positions for the drag. The first item is the start position, the second the last known
    /// position.
    positions: Option<(egui::Pos2, egui::Pos2)>,
}

impl DragState {
    fn reset(&mut self) {
        self.positions.take();
    }

    fn update(&mut self, response: &egui::Response) -> Option<DragEvent> {
        if response.dragged_by(egui::PointerButton::Primary) {
            // Since the response was dragged, this *must* be Some, hence the unwrap.
            let new_pos = response.interact_pointer_pos().unwrap();
            Some(match self.positions {
                None => {
                    self.positions = Some((new_pos, new_pos));
                    DragEvent::Start(new_pos)
                }
                Some((start, ref mut current)) => {
                    *current = new_pos;
                    DragEvent::Update(egui::Rect::from_two_pos(start, *current))
                }
            })
        } else {
            self.positions.take().map(|(start, end)| DragEvent::Finished(egui::Rect::from_two_pos(start, end)))
        }
    }
}
