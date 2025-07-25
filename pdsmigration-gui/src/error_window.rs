use crate::errors::GuiError;
use egui::{InnerResponse, Window};

#[derive(Clone)]
pub struct ErrorWindow {
    open: bool,
    gui_error: GuiError,
}

impl ErrorWindow {
    pub fn new(gui_error: GuiError) -> Self {
        Self {
            open: true,
            gui_error,
        }
    }

    pub fn show(&mut self, ctx: &egui::Context) -> Option<InnerResponse<Option<()>>> {
        Window::new(self.gui_error.to_string())
            .title_bar(false)
            .open(&mut self.open.clone())
            .vscroll(false)
            .resizable(false)
            .show(ctx, |ui| {
                ui.label(format!("{}", self.gui_error));
                let btn = ui.button("Ok");
                if btn.clicked() {
                    self.open = false;
                }
            })
    }

    pub fn open(&self) -> bool {
        self.open
    }

    pub fn gui_error(&self) -> &GuiError {
        &self.gui_error
    }
}
