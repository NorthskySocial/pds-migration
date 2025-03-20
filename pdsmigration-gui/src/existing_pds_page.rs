use crate::agent::login_helper;
use crate::errors::GuiError;
use crate::new_pds_page::NewPdsPage;
use crate::{styles, Page};
use bsky_sdk::BskyAgent;
use egui::Ui;
use std::sync::mpsc::Sender;

pub struct ExistingPdsPage {
    page_tx: Sender<Page>,
    error_tx: Sender<GuiError>,
    old_pds_host: String,
    username: String,
    password: String,
}

impl ExistingPdsPage {
    pub fn new(page_tx: Sender<Page>, error_tx: Sender<GuiError>) -> Self {
        Self {
            page_tx,
            error_tx,
            old_pds_host: "".to_string(),
            username: "".to_string(),
            password: "".to_string(),
        }
    }

    pub fn show(&mut self, ui: &mut Ui) {
        styles::render_subtitle(ui, "Old PDS Login!");

        ui.vertical_centered(|ui| {
            styles::render_input(ui, "Old PDS Host", &mut self.old_pds_host, false);
            styles::render_input(ui, "Handle", &mut self.username, false);
            styles::render_input(ui, "Password", &mut self.password, true);
            styles::render_button(ui, "Submit", || {
                self.old_session_login();
            });
        });
    }

    fn old_session_login(&mut self) {
        let old_pds_host = self.old_pds_host.to_string();
        let username = self.username.to_string();
        let password = self.password.to_string();
        let page_tx = self.page_tx.clone();
        let error_tx = self.error_tx.clone();
        tokio::spawn(async move {
            let bsky_agent = BskyAgent::builder().build().await.unwrap();
            match login_helper(
                &bsky_agent,
                old_pds_host.as_str(),
                username.as_str(),
                password.as_str(),
            )
            .await
            {
                Ok(res) => {
                    let old_pds_token = res.access_jwt.clone();
                    let did = res.did.as_str().to_string();
                    page_tx
                        .send(Page::NewLogin(NewPdsPage::new(
                            page_tx.clone(),
                            error_tx,
                            old_pds_token,
                            old_pds_host,
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
}
