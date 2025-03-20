use crate::Page;
use egui::Ui;
use pdsmigration_common::{CreateAccountApiRequest, ServiceAuthRequest};
use std::sync::mpsc::Sender;

pub struct CreateAccountPage {
    new_pds_host: String,
    invite_code: String,
    new_handle: String,
    new_email: String,
    new_password: String,
    did: String,
    old_pds_token: String,
    old_pds_host: String,

    page_tx: Sender<Page>,
    error_tx: Sender<u32>,
}

impl CreateAccountPage {
    pub fn new(
        page_tx: Sender<Page>,
        error_tx: Sender<u32>,
        did: String,
        old_pds_token: String,
        old_pds_host: String,
    ) -> Self {
        Self {
            new_pds_host: "".to_string(),
            invite_code: "".to_string(),
            new_handle: "".to_string(),
            new_email: "".to_string(),
            new_password: "".to_string(),
            did,
            old_pds_token,
            old_pds_host,
            page_tx,
            error_tx,
        }
    }

    pub fn show(&mut self, ui: &mut Ui) {
        ui.heading("Create Account On New PDS");
        ui.horizontal(|ui| {
            ui.label("New PDS Host: ");
            ui.text_edit_singleline(&mut self.new_pds_host);
        });
        ui.horizontal(|ui| {
            ui.label("Invite Code(Leave Blank if None): ");
            ui.text_edit_singleline(&mut self.invite_code);
        });
        ui.horizontal(|ui| {
            ui.label("Handle on new PDS: ");
            ui.text_edit_singleline(&mut self.new_handle);
        });
        ui.horizontal(|ui| {
            ui.label("Email on new PDS: ");
            ui.text_edit_singleline(&mut self.new_email);
        });
        ui.horizontal(|ui| {
            ui.label("Password on new PDS: ");
            ui.add(egui::TextEdit::singleline(&mut self.new_password).password(true));
        });
        if ui.button("Submit").clicked() {
            self.create_account();
        }
    }

    fn create_account(&self) {
        let did = self.did.clone();
        let token = self.old_pds_token.clone();
        let email = self.new_email.clone();
        let pds_host = self.old_pds_host.clone();
        let new_pds_host = self.new_pds_host.clone();
        let aud = new_pds_host.replace("https://", "did:web:");

        let password = self.new_password.clone();
        let invite_code = self.invite_code.clone();
        let handle = self.new_handle.clone();
        let page_tx = self.page_tx.clone();
        let error_tx = self.error_tx.clone();

        tokio::spawn(async move {
            let service_auth_request = ServiceAuthRequest {
                pds_host: pds_host.clone(),
                aud,
                did: did.clone(),
                token: token.clone(),
            };
            let token = match pdsmigration_common::get_service_auth_api(service_auth_request).await
            {
                Ok(res) => res,
                Err(_) => {
                    // err.send().unwrap();
                    panic!("");
                }
            };

            let create_account_request = CreateAccountApiRequest {
                email,
                handle,
                invite_code,
                password,
                token,
                pds_host: new_pds_host,
                did,
            };
            match pdsmigration_common::create_account_api(create_account_request).await {
                Ok(_) => {
                    // let _ = tx.send(1);
                }
                Err(_) => {
                    // let _ = tx.send(2);
                }
            }
        });
    }

    fn create_account_(&mut self) {
        let did = self.did.clone();
        let token = self.old_pds_token.clone();
        let email = self.new_email.clone();
        let pds_host = self.old_pds_host.clone();
        let new_pds_host = self.new_pds_host.clone();
        let aud = new_pds_host.replace("https://", "did:web:");

        let password = self.new_password.clone();
        let invite_code = self.invite_code.clone();
        let handle = self.new_handle.clone();

        tokio::spawn(async move {
            let service_auth_request = ServiceAuthRequest {
                pds_host: pds_host.clone(),
                aud,
                did: did.clone(),
                token: token.clone(),
            };
            let token = match pdsmigration_common::get_service_auth_api(service_auth_request).await
            {
                Ok(res) => res,
                Err(_) => {
                    // let _ = tx.send(2);
                    panic!("");
                }
            };

            let create_account_request = CreateAccountApiRequest {
                email,
                handle,
                invite_code,
                password,
                token,
                pds_host: new_pds_host,
                did,
            };
            match pdsmigration_common::create_account_api(create_account_request).await {
                Ok(_) => {
                    // let _ = tx.send(1);
                }
                Err(_) => {
                    // let _ = tx.send(2);
                }
            }
        });
    }
}
