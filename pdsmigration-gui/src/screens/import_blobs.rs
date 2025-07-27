use crate::errors::GuiError;
use crate::screens::Screen;
use crate::session::session_config::PdsSession;
use crate::{styles, upload_blobs, ScreenType};
use egui::Ui;
use std::sync::Arc;
use tokio::sync::RwLock;

pub struct ImportBlobs {
    pds_session: Arc<RwLock<PdsSession>>,
    error: Arc<RwLock<Vec<GuiError>>>,
    task_started: bool,
    page: Arc<RwLock<ScreenType>>,
    pds_migration_step: Arc<RwLock<bool>>,
}

impl ImportBlobs {
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
impl Screen for ImportBlobs {
    fn ui(&mut self, ui: &mut Ui, ctx: &egui::Context) {
        styles::render_subtitle(ui, ctx, "Uploading Blobs to new PDS");
        if self.task_started {
            return;
        }
        self.task_started = true;
        let error = self.error.clone();
        let pds_session = {
            let lock = self.pds_session.clone();
            let value = lock.blocking_read();
            value.clone()
        };
        let pds_migration_step = {
            let lock = self.pds_migration_step.clone();
            let value = lock.blocking_read();
            *value
        };
        let page = self.page.clone();
        tokio::spawn(async move {
            tracing::info!("Importing blobs to new PDS");
            match upload_blobs(pds_session).await {
                Ok(_) => {
                    tracing::info!("Importing blobs successful");
                    if pds_migration_step {
                        let mut page_write = page.write().await;
                        *page_write = ScreenType::MigratePreferences;
                    }
                }
                Err(e) => {
                    tracing::error!("Error uploading blobs: {}", e);
                    error.write().await.push(e);
                }
            }
        });
    }

    fn name(&self) -> ScreenType {
        ScreenType::ImportBlobs
    }
}
