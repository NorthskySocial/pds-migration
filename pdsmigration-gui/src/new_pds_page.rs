use crate::agent::login_helper;
use crate::app::Page;
use crate::errors::GuiError;
use crate::home_page::HomePage;
use crate::styles;
use bsky_sdk::BskyAgent;
use egui::Ui;
use pdsmigration_common::{CreateAccountApiRequest, ServiceAuthRequest};
use std::sync::mpsc::{Receiver, Sender};

pub struct NewPdsPage {
    new_pds_host: String,
    new_handle: String,
    new_password: String,
    new_email: String,
    error_tx: Sender<GuiError>,
    page_tx: Sender<Page>,
    success_tx: Sender<String>,
    old_pds_token: String,
    old_pds_refresh: String,
    old_pds_host: String,
    invite_code: String,
    did: String,
    login_tx: Sender<u32>,
    login_rx: Receiver<u32>,
    is_new_account: Option<bool>,
}

impl NewPdsPage {
    pub fn new(
        page_tx: Sender<Page>,
        error_tx: Sender<GuiError>,
        success_tx: Sender<String>,
        old_pds_token: String,
        old_pds_refresh: String,
        old_pds_host: String,
        did: String,
    ) -> Self {
        let (login_tx, login_rx) = std::sync::mpsc::channel();
        Self {
            new_pds_host: "https://northsky.social".to_string(),
            new_handle: "".to_string(),
            page_tx,
            error_tx,
            success_tx,
            new_password: "".to_string(),
            old_pds_token,
            old_pds_refresh,
            old_pds_host,
            invite_code: "".to_string(),
            new_email: "".to_string(),
            did,
            login_tx,
            login_rx,
            is_new_account: None,
        }
    }

    pub fn show(&mut self, ui: &mut Ui) {
        match self.is_new_account {
            None => {
                styles::render_subtitle(ui, "Does your account exist on new PDS?");
                ui.vertical_centered(|ui| {
                    styles::render_button(ui, "Yes", || self.is_new_account = Some(false));
                    styles::render_button(ui, "No", || self.is_new_account = Some(true));
                });
            }
            Some(is_new_account) => {
                if is_new_account {
                    styles::render_subtitle(ui, "Create New PDS Account!");
                    ui.vertical_centered(|ui| {
                        styles::render_input(
                            ui,
                            "New PDS Host",
                            &mut self.new_pds_host,
                            false,
                            Some("https://northsky.social"),
                        );
                        styles::render_input(ui, "Email", &mut self.new_email, false, None);
                        styles::render_input(
                            ui,
                            "Handle",
                            &mut self.new_handle,
                            false,
                            Some("user.northsky.social"),
                        );
                        styles::render_input(ui, "Password", &mut self.new_password, true, None);
                        styles::render_input(
                            ui,
                            "Invite Code (Leave Blank if None)",
                            &mut self.invite_code,
                            false,
                            None,
                        );
                        styles::render_button(ui, "Submit", || {
                            self.create_account();
                        });
                    });
                } else {
                    styles::render_subtitle(ui, "New PDS Login!");
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
                        styles::render_button(ui, "Submit", || {
                            self.new_session_login();
                        });
                    });
                }
            }
        }

        if self.login_rx.try_recv().is_ok() {
            self.new_session_login();
        }
    }

    fn new_session_login(&mut self) {
        let new_pds_host = self.new_pds_host.to_string();
        let new_handle = self.new_handle.to_string();
        let new_password = self.new_password.to_string();
        let page_tx = self.page_tx.clone();
        let error_tx = self.error_tx.clone();
        let success_tx = self.success_tx.clone();
        let old_pds_token = self.old_pds_token.clone();
        let old_pds_host = self.old_pds_host.clone();

        tokio::spawn(async move {
            let bsky_agent = BskyAgent::builder().build().await.unwrap();
            match login_helper(
                &bsky_agent,
                new_pds_host.as_str(),
                new_handle.as_str(),
                new_password.as_str(),
            )
            .await
            {
                Ok(res) => {
                    let new_pds_token = res.access_jwt.clone();
                    let did = res.did.as_str().to_string();
                    page_tx
                        .send(Page::Home(HomePage::new(
                            page_tx.clone(),
                            error_tx,
                            success_tx,
                            old_pds_token,
                            new_pds_token,
                            old_pds_host,
                            new_pds_host,
                            did,
                        )))
                        .unwrap();
                }
                Err(e) => {
                    error_tx.send(e).unwrap();
                }
            };
        });
    }

    fn create_account(&mut self) {
        let did = self.did.clone();
        let token = self.old_pds_token.clone();
        let email = self.new_email.clone();
        let pds_host = self.old_pds_host.clone();
        let new_pds_host = self.new_pds_host.clone();
        let aud = new_pds_host.replace("https://", "did:web:");

        let password = self.new_password.clone();
        let invite_code = self.invite_code.clone();
        let handle = self.new_handle.clone();

        let error_tx = self.error_tx.clone();
        let login_tx = self.login_tx.clone();

        tokio::spawn(async move {
            let service_auth_request = ServiceAuthRequest {
                pds_host: pds_host.clone(),
                aud,
                did: did.clone(),
                token: token.clone(),
            };
            let service_token =
                match pdsmigration_common::get_service_auth_api(service_auth_request).await {
                    Ok(res) => res,
                    Err(_pds_error) => {
                        error_tx.send(GuiError::Runtime).unwrap();
                        return;
                    }
                };

            let create_account_request = CreateAccountApiRequest {
                email,
                handle,
                invite_code,
                password,
                token: service_token,
                pds_host: new_pds_host,
                did,
                recovery_key: None,
            };
            match pdsmigration_common::create_account_api(create_account_request).await {
                Ok(_) => {
                    login_tx.send(1).unwrap();
                }
                Err(_pds_error) => {
                    error_tx.send(GuiError::Runtime).unwrap();
                }
            }
        });
    }
}
