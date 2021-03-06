use std::borrow::Cow;
use crate::egui;

#[derive(Clone, Debug, Default, serde::Serialize, serde::Deserialize)]
#[must_use = "You should call .store()"]
struct State {
    selection: String,
}

impl State {
    const ID: &'static str = "entities";

    pub fn load(ctx: &egui::Context) -> Option<Self> {
        ctx.data().get_persisted(egui::Id::new(Self::ID))
    }

    pub fn store(self, ctx: &egui::Context) {
        ctx.data()
            .insert_persisted(egui::Id::new(Self::ID), self);
    }
}

pub struct Entities<'a> {
    entities: &'a mut crate::model::entities::Entities,
}

impl<'a> Entities<'a> {
    pub fn new(entities: &'a mut crate::model::entities::Entities) -> Self {
        Self { entities }
    }
}

impl Entities<'_> {
    /// Shows the widget.
    ///
    /// # Arguments
    ///
    /// * `ui`: The [`Ui`](egui::Ui).
    ///
    /// returns: the selected [`Entity`] or `None` if no [`Entity`] was selected.
    pub fn show(&mut self, ui: &mut egui::Ui) -> Option<Cow<'static, str>> {
        let mut state = State::load(ui.ctx()).unwrap_or_default();

        let mut out = None;
        for (name, _) in self.entities.entries() {
            let name_str: &str = &*name;
            let selected = name_str == &state.selection;
            if selected {
                out.replace(name.clone());
            }
            if ui.radio(selected, name_str).clicked() {
                state.selection = name.clone().into_owned();
            }
        }

        state.store(ui.ctx());
        out
    }
}