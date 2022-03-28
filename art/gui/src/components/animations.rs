use ves_art_core::sprite::Animation;
use crate::egui;

#[derive(Clone, Debug, Default, serde::Serialize, serde::Deserialize)]
#[must_use = "You should call .store()"]
struct State {
    selection: String,
}

impl State {
    const ID: &'static str = "animations";

    pub fn load(ctx: &egui::Context) -> Option<Self> {
        ctx.data().get_persisted(egui::Id::new(Self::ID))
    }

    pub fn store(self, ctx: &egui::Context) {
        ctx.data()
            .insert_persisted(egui::Id::new(Self::ID), self);
    }
}

pub struct Animations<'a> {
    animations: &'a mut crate::model::entities::Animations,
}

impl<'a> Animations<'a> {
    pub fn new(animations: &'a mut crate::model::entities::Animations) -> Self {
        Self { animations }
    }
}

impl Animations<'_> {
    /// Shows the widget.
    ///
    /// # Arguments
    ///
    /// * `ui`: The [`Ui`](egui::Ui).
    ///
    /// returns: the selected [`Animation`] or `None` if no [`Animation`] was selected.
    pub fn show(&mut self, ui: &mut egui::Ui) -> Option<&Animation> {
        let mut state = State::load(ui.ctx()).unwrap_or_default();

        let mut out = None;
        for (name, animation) in self.animations.entries() {
            let name_str: &str = &*name;
            let selected = name_str == &state.selection;
            if selected {
                out.replace(animation);
            }
            if ui.radio(selected, name_str).clicked() {
                state.selection = name.clone().into_owned();
            }
        }

        state.store(ui.ctx());
        out
    }
}