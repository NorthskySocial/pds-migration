use crate::errors::GuiError;
use crate::screens::Screen;
use crate::session::session_config::PdsSession;
use crate::{export_missing_blobs, styles, ScreenType};
use egui::Ui;
use std::sync::Arc;
use tokio::sync::RwLock;

pub struct ExportBlobs {
    pds_session: Arc<RwLock<PdsSession>>,
    error: Arc<RwLock<Vec<GuiError>>>,
    task_started: bool,
    page: Arc<RwLock<ScreenType>>,
    pds_migration_step: Arc<RwLock<bool>>,
}

impl ExportBlobs {
    pub fn new(
        pds_session: Arc<RwLock<PdsSession>>,
        error: Arc<RwLock<Vec<GuiError>>>,
        page: Arc<RwLock<ScreenType>>,
        pds_migration_step: Arc<RwLock<bool>>,
    ) -> Self {
        Self {
            pds_session,
            error,
            task_started: false,
            page,
            pds_migration_step,
        }
    }
}
impl Screen for ExportBlobs {
    #[tracing::instrument(skip(self, ui, ctx))]
    fn ui(&mut self, ui: &mut Ui, ctx: &egui::Context) {
        styles::render_subtitle(ui, ctx, "Exporting blobs from old PDS");
        if self.task_started {
            return;
        }
        self.task_started = true;

        let pds_session = {
            let lock = self.pds_session.clone();
            let value = lock.try_read();
            match value {
                Ok(value) => value.clone(),
                Err(_) => {
                    tracing::debug!("Unable to read pds session");
                    self.task_started = false;
                    return;
                }
            }
        };
        let error = self.error.clone();
        let pds_migration_step = {
            let lock = self.pds_migration_step.clone();
            let value = lock.try_read();
            match value {
                Ok(value) => *value,
                Err(_) => {
                    tracing::debug!("Unable to read pds migration step");
                    self.task_started = false;
                    return;
                }
            }
        };
        let page = self.page.clone();
        tokio::spawn(async move {
            tracing::info!("Exporting blobs from old PDS");
            match export_missing_blobs(pds_session).await {
                Ok(_) => {
                    tracing::info!("Blobs exported from old PDS");
                    if pds_migration_step {
                        let mut page_write = page.write().await;
                        *page_write = ScreenType::ImportRepo;
                    }
                }
                Err(e) => {
                    tracing::error!("Error exporting blobs from old PDS: {}", e);
                    let mut error_write = error.write().await;
                    error_write.push(e);
                }
            }
        });
    }

    fn name(&self) -> ScreenType {
        ScreenType::ExportBlobs
    }
}
