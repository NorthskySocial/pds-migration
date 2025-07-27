use crate::errors::GuiError;
use crate::screens::Screen;
use crate::session::session_config::PdsSession;
use crate::{migrate_preferences, styles, ScreenType};
use egui::Ui;
use std::sync::Arc;
use tokio::sync::RwLock;

pub struct MigratePreferences {
    pds_session: Arc<RwLock<PdsSession>>,
    error: Arc<RwLock<Vec<GuiError>>>,
    task_started: bool,
    page: Arc<RwLock<ScreenType>>,
}

impl MigratePreferences {
    pub fn new(
        pds_session: Arc<RwLock<PdsSession>>,
        error: Arc<RwLock<Vec<GuiError>>>,
        page: Arc<RwLock<ScreenType>>,
        _pds_migration_step: Arc<RwLock<bool>>,
    ) -> Self {
        Self {
            pds_session,
            error,
            task_started: false,
            page,
        }
    }
}
impl Screen for MigratePreferences {
    fn ui(&mut self, ui: &mut Ui, ctx: &egui::Context) {
        styles::render_subtitle(ui, ctx, "Migrating preferences");
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
        let page = self.page.clone();
        tokio::spawn(async move {
            tracing::info!("Migrating preferences from old to new PDS");
            match migrate_preferences(pds_session).await {
                Ok(_) => {
                    tracing::info!("Preferences migrated successfully");
                    let mut page_write = page.write().await;
                    *page_write = ScreenType::MigratePLC;
                }
                Err(e) => {
                    let mut errors = error.write().await;
                    errors.push(e);
                }
            }
        });
    }

    fn name(&self) -> ScreenType {
        ScreenType::MigratePreferences
    }
}
