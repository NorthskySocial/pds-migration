use crate::errors::GuiError;
use crate::screens::Screen;
use crate::session::session_config::PdsSession;
use crate::{export_repo, styles, ScreenType};
use egui::Ui;
use std::sync::Arc;
use tokio::sync::RwLock;

pub struct ExportRepo {
    pds_session: Arc<RwLock<PdsSession>>,
    error: Arc<RwLock<Vec<GuiError>>>,
    task_started: bool,
    page: Arc<RwLock<ScreenType>>,
}

impl ExportRepo {
    pub fn new(
        pds_session: Arc<RwLock<PdsSession>>,
        error: Arc<RwLock<Vec<GuiError>>>,
        page: Arc<RwLock<ScreenType>>,
    ) -> Self {
        Self {
            pds_session,
            error,
            task_started: false,
            page,
        }
    }
}
impl Screen for ExportRepo {
    fn ui(&mut self, ui: &mut Ui, ctx: &egui::Context) {
        styles::render_subtitle(ui, ctx, "Exporting Repo from old PDS");
        if self.task_started {
            return;
        }
        let pds_session = {
            let lock = self.pds_session.clone();
            let value = lock.blocking_read();
            value.clone()
        };
        let error = self.error.clone();
        let page = self.page.clone();
        tokio::spawn(async move {
            tracing::info!("Exporting repo from old PDS");
            match export_repo(pds_session).await {
                Ok(_) => {
                    tracing::info!("Repo exported successfully");
                    let mut page = page.write().await;
                    *page = ScreenType::ExportBlobs;
                }
                Err(e) => {
                    let mut errors = error.write().await;
                    errors.push(e);
                }
            }
        });
    }

    fn name(&self) -> ScreenType {
        ScreenType::ExportRepo
    }
}
