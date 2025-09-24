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
    task_started: Arc<RwLock<bool>>,
    page: Arc<RwLock<ScreenType>>,
}

impl ExportRepo {
    pub fn new(
        pds_session: Arc<RwLock<PdsSession>>,
        error: Arc<RwLock<Vec<GuiError>>>,
        page: Arc<RwLock<ScreenType>>,
    ) -> Self {
        let task_started = Arc::new(RwLock::new(false));
        Self {
            pds_session,
            error,
            task_started,
            page,
        }
    }
}

impl Screen for ExportRepo {
    fn ui(&mut self, ui: &mut Ui, ctx: &egui::Context) {
        styles::render_subtitle(ui, ctx, "Exporting Repo from old PDS");
        let task_started = self.task_started.blocking_read();
        if *task_started {
            return;
        }
        drop(task_started);

        let pds_session = {
            let lock = self.pds_session.clone();
            let value = lock.blocking_read();
            value.clone()
        };
        let error = self.error.clone();
        let page = self.page.clone();
        let task_writer = self.task_started.clone();
        tokio::spawn(async move {
            tracing::info!("Exporting repo from old PDS");
            let mut writer = task_writer.write().await;
            *writer = true;
            drop(writer);
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
