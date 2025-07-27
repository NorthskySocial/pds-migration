use crate::agent::login_helper2;
use crate::errors::GuiError;
use crate::screens::Screen;
use crate::session::session_config::PdsSession;
use crate::{styles, ScreenType};
use bsky_sdk::BskyAgent;
use egui::Ui;
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Clone)]
pub struct NewLogin {
    new_pds_host: String,
    new_handle: String,
    new_password: String,
    pds_session: Arc<RwLock<PdsSession>>,
    error: Arc<RwLock<Vec<GuiError>>>,
    page: Arc<RwLock<ScreenType>>,
    pds_migration_step: Arc<RwLock<bool>>,
}

impl NewLogin {
    pub fn new(
        pds_session: Arc<RwLock<PdsSession>>,
        error: Arc<RwLock<Vec<GuiError>>>,
        page: Arc<RwLock<ScreenType>>,
        pds_migration_step: Arc<RwLock<bool>>,
    ) -> Self {
        Self {
            new_pds_host: "".to_string(),
            new_handle: "".to_string(),
            new_password: "".to_string(),
            pds_session,
            error,
            page,
            pds_migration_step,
        }
    }

    #[tracing::instrument(skip(self))]
    fn new_session_login(&mut self) {
        let new_pds_host = self.new_pds_host.to_string();
        let new_handle = self.new_handle.to_string();
        let new_password = self.new_password.to_string();
        let pds_session_lock = self.pds_session.clone();
        let error_lock = self.error.clone();
        let page_lock = self.page.clone();
        let pds_migration_step_lock = self.pds_migration_step.clone();

        tokio::spawn(async move {
            let bsky_agent = BskyAgent::builder().build().await.unwrap();
            match login_helper2(
                &bsky_agent,
                new_pds_host.as_str(),
                new_handle.as_str(),
                new_password.as_str(),
            )
            .await
            {
                Ok(res) => {
                    tracing::info!("Login successful");
                    let access_token = res.access_jwt.clone();
                    let refresh_token = res.refresh_jwt.clone();
                    let did = res.did.as_str().to_string();
                    {
                        let mut pds_session = pds_session_lock.write().await;
                        pds_session.create_new_session(
                            did.as_str(),
                            access_token.as_str(),
                            refresh_token.as_str(),
                            new_pds_host.as_str(),
                        );
                    }
                    let pds_migration_step = {
                        let value = pds_migration_step_lock.read().await;
                        *value
                    };
                    if pds_migration_step {
                        let mut page = page_lock.write().await;
                        *page = ScreenType::ExportRepo;
                    } else {
                        let mut page = page_lock.write().await;
                        *page = ScreenType::Basic;
                    }
                }
                Err(e) => {
                    tracing::error!("Error logging in: {e}");
                    let mut error = error_lock.write().await;
                    error.push(GuiError::Other);
                }
            };
        });
    }
}
impl Screen for NewLogin {
    fn ui(&mut self, ui: &mut Ui, ctx: &egui::Context) {
        styles::render_subtitle(ui, ctx, "New PDS Login!");
        ui.vertical_centered(|ui| {
            styles::render_input(
                ui,
                "New PDS Host",
                &mut self.new_pds_host,
                false,
                Some("https://northsky.social"),
            );
            styles::render_input(
                ui,
                "Handle",
                &mut self.new_handle,
                false,
                Some("user.northsky.social"),
            );
            styles::render_input(ui, "Password", &mut self.new_password, true, None);
            styles::render_button(ui, ctx, "Submit", || {
                self.new_session_login();
            });
        });
    }

    fn name(&self) -> ScreenType {
        ScreenType::NewLogin
    }
}
