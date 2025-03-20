use crate::agent::login_helper;
use crate::errors::GuiError;
use crate::home_page::HomePage;
use crate::{styles, Page};
use bsky_sdk::BskyAgent;
use egui::Ui;
use std::sync::mpsc::Sender;

pub struct NewPdsPage {
    new_pds_host: String,
    new_handle: String,
    new_password: String,
    error_tx: Sender<GuiError>,
    page_tx: Sender<Page>,
    old_pds_token: String,
    old_pds_host: String,
}

impl NewPdsPage {
    pub fn new(
        page_tx: Sender<Page>,
        error_tx: Sender<GuiError>,
        old_pds_token: String,
        old_pds_host: String,
    ) -> Self {
        Self {
            new_pds_host: "".to_string(),
            new_handle: "".to_string(),
            page_tx,
            error_tx,
            new_password: "".to_string(),
            old_pds_token,
            old_pds_host,
        }
    }

    pub fn show(&mut self, ui: &mut Ui) {
        ui.vertical_centered(|ui| {
            styles::render_input(ui, "New PDS URL", &mut self.new_pds_host, false);
            styles::render_input(ui, "Username", &mut self.new_handle, false);
            styles::render_input(ui, "Password", &mut self.new_password, true);
            styles::render_button(ui, "Submit", || {
                self.new_session_login();
            });
        });
    }

    fn new_session_login(&mut self) {
        let new_pds_host = self.new_pds_host.to_string();
        let new_handle = self.new_handle.to_string();
        let new_password = self.new_password.to_string();
        let page_tx = self.page_tx.clone();
        let error_tx = self.error_tx.clone();
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
}
