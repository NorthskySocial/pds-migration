use crate::errors::GuiError;
use crate::screens::Screen;
use crate::session::session_config::PdsSession;
use crate::{export_all_blobs, export_repo, styles, ScreenType};
use egui::{ScrollArea, Ui};
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Clone)]
pub struct BasicHome {
    pds_session: Arc<RwLock<PdsSession>>,
    error: Arc<RwLock<Vec<GuiError>>>,
    page: Arc<RwLock<ScreenType>>,
    pds_migration_step: Arc<RwLock<bool>>,
}

impl BasicHome {
    pub fn new(
        pds_session: Arc<RwLock<PdsSession>>,
        error: Arc<RwLock<Vec<GuiError>>>,
        page: Arc<RwLock<ScreenType>>,
        pds_migration_step: Arc<RwLock<bool>>,
    ) -> Self {
        Self {
            pds_session,
            error,
            page,
            pds_migration_step,
        }
    }

    pub fn show_logged_in(&self, ui: &mut Ui, ctx: &egui::Context) {
        ScrollArea::both().show(ui, |ui| {
            styles::render_button(ui, ctx, "Migrate to new PDS", || {
                let pds_migration_step = self.pds_migration_step.clone();
                let page_lock = self.page.clone();
                tokio::spawn(async move {
                    let mut pds_migration_step = pds_migration_step.write().await;
                    *pds_migration_step = true;
                    let mut page = page_lock.write().await;
                    *page = ScreenType::DoesAccountExist;
                });
            });
            styles::render_button(ui, ctx, "Backup Repo", || {
                let pds_session = {
                    let pds_session_lock = self.pds_session.clone();
                    let value = pds_session_lock.blocking_read();
                    value.clone()
                };
                let error = self.error.clone();
                tokio::spawn(async move {
                    match export_repo(pds_session).await {
                        Ok(_) => {
                            tracing::info!("Repo exported successfully");
                        }
                        Err(e) => {
                            let mut error = error.write().await;
                            error.push(e);
                        }
                    }
                });
            });
            styles::render_button(ui, ctx, "Backup Media", || {
                let pds_session = {
                    let pds_session_lock = self.pds_session.clone();
                    let value = pds_session_lock.blocking_read();
                    value.clone()
                };
                let error = self.error.clone();
                tokio::spawn(async move {
                    match export_all_blobs(pds_session).await {
                        Ok(_) => {}
                        Err(e) => {
                            let mut error = error.write().await;
                            error.push(e);
                        }
                    }
                });
            });
        });
    }

    pub fn show_logged_out(&self, ui: &mut Ui, _ctx: &egui::Context) {
        ScrollArea::both().show(ui, |_ui| {
            // styles::render_button(ui, ctx, "Migrate to PDS without a PDS", || {
            //     let pds_migration_step = self.pds_migration_step.clone();
            //     let page_lock = self.page.clone();
            //     tokio::spawn(async move {
            //         let mut pds_migration_step = pds_migration_step.write().await;
            //         *pds_migration_step = true;
            //         let mut page = page_lock.write().await;
            //         *page = ScreenType::MigrateWithoutPds;
            //     });
            // });
        });
    }
}

impl Screen for BasicHome {
    fn ui(&mut self, ui: &mut Ui, ctx: &egui::Context) {
        ScrollArea::both().show(ui, |ui| {
            let pds_session = { self.pds_session.blocking_read().clone() };
            if pds_session.old_session_config().is_some() {
                self.show_logged_in(ui, ctx);
            } else {
                self.show_logged_out(ui, ctx);
            }
        });
    }

    fn name(&self) -> ScreenType {
        ScreenType::Basic
    }
}
