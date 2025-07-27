use crate::screens::Screen;
use crate::{styles, ScreenType};
use egui::Ui;
use std::sync::Arc;
use tokio::sync::RwLock;

pub struct Success {
    page: Arc<RwLock<ScreenType>>,
    pds_migration_step: Arc<RwLock<bool>>,
}

impl Success {
    pub fn new(page: Arc<RwLock<ScreenType>>, pds_migration_step: Arc<RwLock<bool>>) -> Self {
        Self {
            page,
            pds_migration_step,
        }
    }
}
impl Screen for Success {
    fn ui(&mut self, ui: &mut Ui, ctx: &egui::Context) {
        styles::render_subtitle(ui, ctx, "Congratulations, you are successfully migrated!");
        styles::render_button(ui, ctx, "Home", || {
            let page_lock = self.page.clone();
            let pds_migration_step_lock = self.pds_migration_step.clone();
            tokio::spawn(async move {
                let mut pds_migration_step_write = pds_migration_step_lock.write().await;
                *pds_migration_step_write = false;
                let mut page_write = page_lock.write().await;
                *page_write = ScreenType::Basic;
            });
        });
    }

    fn name(&self) -> ScreenType {
        ScreenType::Success
    }
}
