use egui::{InnerResponse, Window};

#[derive(Clone)]
pub struct SuccessWindow {
    open: bool,
    message: String,
}

impl SuccessWindow {
    pub fn new(message: String) -> Self {
        Self {
            open: true,
            message,
        }
    }

    pub fn show(&mut self, ctx: &egui::Context) -> Option<InnerResponse<Option<()>>> {
        Window::new(self.message.clone())
            .title_bar(false)
            .open(&mut self.open.clone())
            .vscroll(false)
            .resizable(false)
            .show(ctx, |ui| {
                ui.label(self.message.to_string());
                let btn = ui.button("Ok");
                if btn.clicked() {
                    self.open = false;
                }
            })
    }

    pub fn open(&self) -> bool {
        self.open
    }

    pub fn message(&self) -> &str {
        &self.message
    }
}
