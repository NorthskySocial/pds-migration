use crate::errors::GuiError;
use crate::screens::Screen;
use crate::session::session_config::PdsSession;
use crate::{export_repo, styles, ScreenType};
use egui::{ScrollArea, Ui};
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Clone)]
pub struct AdvancedHome {
    pds_session: Arc<RwLock<PdsSession>>,
    error: Arc<RwLock<Vec<GuiError>>>,
    page: Arc<RwLock<ScreenType>>,
}

impl AdvancedHome {
    pub fn new(
        pds_session: Arc<RwLock<PdsSession>>,
        error: Arc<RwLock<Vec<GuiError>>>,
        page: Arc<RwLock<ScreenType>>,
    ) -> Self {
        Self {
            pds_session,
            error,
            page,
        }
    }
}

impl Screen for AdvancedHome {
    fn ui(&mut self, ui: &mut Ui, ctx: &egui::Context) {
        ScrollArea::both().show(ui, |ui| {
            styles::render_button(ui, ctx, "Export Repo", || {
                let pds_session = {
                    let pds_session_lock = self.pds_session.clone();
                    let value = pds_session_lock.blocking_read();
                    value.clone()
                };
                let error = self.error.clone();
                tokio::spawn(async move {
                    match export_repo(pds_session).await {
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

    fn name(&self) -> ScreenType {
        ScreenType::Advanced
    }
}
