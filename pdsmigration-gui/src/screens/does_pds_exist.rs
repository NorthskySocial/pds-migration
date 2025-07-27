use crate::screens::Screen;
use crate::{styles, ScreenType};
use egui::Ui;
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Clone)]
pub struct DoesPdsExist {
    page: Arc<RwLock<ScreenType>>,
}

impl DoesPdsExist {
    pub fn new(page: Arc<RwLock<ScreenType>>) -> Self {
        Self { page }
    }
}
impl Screen for DoesPdsExist {
    fn ui(&mut self, ui: &mut Ui, ctx: &egui::Context) {
        styles::render_subtitle(ui, ctx, "Does your account exist on new PDS?");
        ui.vertical_centered(|ui| {
            styles::render_button(ui, ctx, "Yes", || {
                let mut page_write = self.page.blocking_write();
                *page_write = ScreenType::NewLogin
            });
            styles::render_button(ui, ctx, "No", || {
                let mut page_write = self.page.blocking_write();
                *page_write = ScreenType::CreateNewAccount
            });
        });
    }

    fn name(&self) -> ScreenType {
        ScreenType::DoesAccountExist
    }
}
