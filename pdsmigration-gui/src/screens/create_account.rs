use crate::errors::GuiError;
use crate::screens::Screen;
use crate::session::session_config::PdsSession;
use crate::{
    create_account, fetch_tos_and_privacy_policy, styles, CreateAccountParameters, ScreenType,
};
use egui::Ui;
use std::sync::Arc;
use tokio::sync::RwLock;

pub struct CreateAccount {
    new_pds_host: String,
    new_handle: String,
    new_password: String,
    confirm_password: String,
    pds_session: Arc<RwLock<PdsSession>>,
    error: Arc<RwLock<Vec<GuiError>>>,
    pds_selected: bool,
    new_email: String,
    invite_code: String,
    privacy_policy_lock: Arc<RwLock<Option<String>>>,
    terms_of_service_lock: Arc<RwLock<Option<String>>>,
    invite_code_required: Arc<RwLock<bool>>,
    available_user_domains: Arc<RwLock<Vec<String>>>,
    page: Arc<RwLock<ScreenType>>,
    pds_migration_step: Arc<RwLock<bool>>,
}

impl CreateAccount {
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
            confirm_password: "".to_string(),
            pds_session,
            error,
            pds_selected: false,
            new_email: "".to_string(),
            invite_code: "".to_string(),
            privacy_policy_lock: Arc::new(Default::default()),
            terms_of_service_lock: Arc::new(Default::default()),
            invite_code_required: Arc::new(Default::default()),
            available_user_domains: Arc::new(Default::default()),
            page,
            pds_migration_step,
        }
    }

    fn update_pds(&mut self) {
        let error = self.error.clone();
        let terms_of_service_lock = self.terms_of_service_lock.clone();
        let privacy_policy_lock = self.privacy_policy_lock.clone();
        let invite_code_required = self.invite_code_required.clone();
        let available_user_domains = self.available_user_domains.clone();
        let new_pds_host = self.new_pds_host.clone();
        tokio::spawn(async move {
            match fetch_tos_and_privacy_policy(new_pds_host).await {
                Ok(result) => {
                    let mut privacy_policy_write = privacy_policy_lock.write().await;
                    *privacy_policy_write = result.privacy_policy;
                    let mut terms_of_service_lock = terms_of_service_lock.write().await;
                    *terms_of_service_lock = result.terms_of_service;
                    let mut invite_code_required_write = invite_code_required.write().await;
                    *invite_code_required_write = result.invite_code_required;
                    let mut available_user_domains_write = available_user_domains.write().await;
                    *available_user_domains_write = result.available_user_domains;
                }
                Err(e) => {
                    let mut errors = error.write().await;
                    errors.push(e);
                }
            }
        });
    }

    fn submit(&mut self) {
        let pds_session = {
            let lock = self.pds_session.clone();
            let value = lock.blocking_read();
            value.clone()
        };
        let new_email = self.new_email.clone();
        let new_pds_host = self.new_pds_host.clone();
        let new_password = self.new_password.clone();
        let new_handle = self.new_handle.clone();
        let invite_code = self.invite_code.clone();
        let params = CreateAccountParameters {
            pds_session,
            new_email,
            new_pds_host,
            new_password,
            new_handle,
            invite_code,
        };
        let error = self.error.clone();
        let pds_session_lock = self.pds_session.clone();
        let page = self.page.clone();
        let pds_migration_step = self.pds_migration_step.clone();

        tokio::spawn(async move {
            match create_account(params).await {
                Ok(pds_session) => {
                    {
                        let mut pds_session_write = pds_session_lock.write().await;
                        *pds_session_write = pds_session;
                    }
                    let pds_migration_step_read = { *pds_migration_step.read().await };
                    match pds_migration_step_read {
                        false => {
                            let mut page_write = page.write().await;
                            *page_write = ScreenType::Basic;
                        }
                        true => {
                            let mut page_write = page.write().await;
                            *page_write = ScreenType::ExportRepo;
                        }
                    }
                }
                Err(e) => {
                    let mut errors = error.write().await;
                    errors.push(e);
                }
            }
        });
    }
}

impl Screen for CreateAccount {
    fn ui(&mut self, ui: &mut Ui, ctx: &egui::Context) {
        let _handle = self.new_handle.clone();
        let available_user_domains = {
            let available_user_domains = self.available_user_domains.blocking_read();
            available_user_domains
                .get(0)
                .cloned()
                .unwrap_or("".to_string())
        };
        styles::render_subtitle(ui, ctx, "Create New PDS Account!");
        ui.vertical_centered(|ui| {
            ui.horizontal(|ui| {
                if self.pds_selected {
                    styles::render_input_disabled(
                        ui,
                        "New PDS Host",
                        &mut self.new_pds_host,
                        false,
                        Some("https://northsky.social"),
                    );
                    styles::render_button(ui, ctx, "Edit", || self.pds_selected = false);
                } else {
                    styles::render_input(
                        ui,
                        "New PDS Host",
                        &mut self.new_pds_host,
                        false,
                        Some("https://northsky.social"),
                    );
                    styles::render_button(ui, ctx, "Update", || {
                        self.pds_selected = true;
                        self.update_pds();
                    });
                }
            });
            if self.pds_selected {
                styles::render_input(ui, "Email", &mut self.new_email, false, None);
                styles::render_input(
                    ui,
                    "Handle",
                    &mut self.new_handle,
                    false,
                    Some("user.northsky.social"),
                );
                ui.label(format!(
                    "If not using a custom domain, please append with {available_user_domains}"
                ));
                styles::render_input(ui, "Password", &mut self.new_password, true, None);
                styles::render_input(ui, "Password", &mut self.confirm_password, true, None);
                styles::render_input(
                    ui,
                    "Invite Code (Leave Blank if None)",
                    &mut self.invite_code,
                    false,
                    None,
                );

                let privacy_policy = {
                    let privacy_policy = self.privacy_policy_lock.blocking_read();
                    let value = privacy_policy.clone();
                    value.unwrap_or("".to_string())
                };
                let terms_of_service = {
                    let terms_of_service = self.privacy_policy_lock.blocking_read().clone();
                    let value = terms_of_service.clone();
                    value.unwrap_or("".to_string())
                };
                if !privacy_policy.is_empty() || !terms_of_service.is_empty() {
                    ui.horizontal_wrapped(|ui| {
                        ui.spacing_mut().item_spacing.x = 0.0;
                        ui.label("By creating an account you agree to the ");

                        if !terms_of_service.is_empty() {
                            ui.hyperlink_to("Terms of Service", terms_of_service);
                            if !privacy_policy.is_empty() {
                                ui.label(" and ");
                                ui.hyperlink_to("Privacy Policy", privacy_policy);
                                ui.label(".");
                            } else {
                                ui.label(".");
                            }
                        } else {
                            ui.hyperlink_to("Privacy Policy", privacy_policy);
                            ui.label(".");
                        }
                    });
                }
                styles::render_button(ui, ctx, "Submit", || {
                    self.submit();
                });
            }
        });
    }

    fn name(&self) -> ScreenType {
        ScreenType::CreateNewAccount
    }
}
