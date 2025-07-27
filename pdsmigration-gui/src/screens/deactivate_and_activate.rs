use crate::errors::GuiError;
use crate::screens::Screen;
use crate::session::session_config::PdsSession;
use crate::{activate_account, deactivate_account, styles, ScreenType};
use egui::Ui;
use std::sync::Arc;
use tokio::sync::RwLock;

pub struct DeactivateAndActivate {
    pds_session: Arc<RwLock<PdsSession>>,
    error: Arc<RwLock<Vec<GuiError>>>,
    task_started: bool,
    page: Arc<RwLock<ScreenType>>,
}

impl DeactivateAndActivate {
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

    fn deactivate_and_activate(&mut self) {
        let error = self.error.clone();
        let pds_session = {
            let lock = self.pds_session.clone();
            let value = lock.blocking_read();
            value.clone()
        };
        let old_session_config = match pds_session.old_session_config() {
            None => {
                let mut error_write = error.blocking_write();
                error_write.push(GuiError::Other);
                return;
            }
            Some(config) => config.clone(),
        };
        let new_session_config = match pds_session.new_session_config() {
            None => {
                let mut error_write = error.blocking_write();
                error_write.push(GuiError::Other);
                return;
            }
            Some(config) => config.clone(),
        };
        let page = self.page.clone();
        tokio::spawn(async move {
            tracing::info!("Deactivating old account, and activating new account");
            match activate_account(new_session_config).await {
                Ok(_) => {
                    tracing::info!("Activated new account");
                }
                Err(e) => {
                    let mut error_write = error.write().await;
                    error_write.push(e);
                }
            }
            match deactivate_account(old_session_config).await {
                Ok(_) => {
                    tracing::info!("Deactivated old account");
                }
                Err(e) => {
                    let mut error_write = error.write().await;
                    error_write.push(e);
                }
            }

            let mut page_write = page.write().await;
            *page_write = ScreenType::Success;
        });
    }
}

impl Screen for DeactivateAndActivate {
    fn ui(&mut self, ui: &mut Ui, ctx: &egui::Context) {
        styles::render_subtitle(
            ui,
            ctx,
            "Deactivating old account, and activating new account",
        );
        if self.task_started {
            return;
        }
        self.task_started = true;
        self.deactivate_and_activate()
    }

    fn name(&self) -> ScreenType {
        ScreenType::ActiveAccounts
    }
}
