use crate::egui;

#[must_use = "You should call .show()"]
pub struct Window<'a> {
    inner: egui::Window<'a>,
}

impl Window<'_> {
    pub fn new(title: impl Into<egui::WidgetText>) -> Self {
        Self {
            inner: egui::Window::new(title).auto_sized().collapsible(false),
        }
    }

    #[inline]
    pub fn show<R>(
        self,
        ctx: &egui::Context,
        add_contents: impl FnOnce(&mut egui::Ui) -> R,
    ) -> Option<egui::InnerResponse<Option<R>>> {
        self.inner.show(ctx, add_contents)
    }
}
