use crate::agent::login_helper2;
use crate::errors::GuiError;
use crate::session_config::PdsSession;
use crate::styles;
use bsky_sdk::BskyAgent;
use egui::{Align, CollapsingHeader, Color32, InnerResponse, Layout, RichText, Ui, Window};
use multibase::Base::Base58Btc;
use pdsmigration_common::agent::login_helper;
use pdsmigration_common::errors::PdsError;
use pdsmigration_common::{
    ActivateAccountRequest, CreateAccountApiRequest, DeactivateAccountRequest, ExportBlobsRequest,
    ExportPDSRequest, ImportPDSRequest, MigratePlcRequest, MigratePreferencesRequest,
    RequestTokenRequest, ServiceAuthRequest, UploadBlobsRequest,
};
use secp256k1::Secp256k1;
use std::io::Write;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::Arc;
use tokio::sync::RwLock;
use zip::write::SimpleFileOptions;
use zip::{AesMode, ZipWriter};

#[derive(PartialEq)]
pub enum Page {
    Home,
    OldLogin,
    NewLogin,
}

pub struct PdsMigrationApp {
    page: Page,
    error_windows: Vec<ErrorWindow>,
    success_windows: Vec<SuccessWindow>,
    pds_session: Arc<RwLock<PdsSession>>,
    old_pds_host: String,
    username: String,
    password: String,
    user_recovery_key_password: String,
    plc_token: String,
    user_recovery_key: String,
    new_password: String,
    new_pds_host: String,
    new_email: String,
    new_handle: String,
    invite_code: String,
    is_new_account: Option<bool>,
    error_rx: Receiver<GuiError>,
    error_tx: Sender<GuiError>,
    success_rx: Receiver<String>,
    success_tx: Sender<String>,
    page_rx: Receiver<Page>,
    page_tx: Sender<Page>,
}

impl PdsMigrationApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        Default::default()
    }

    // Helper function to create consistent navigation buttons
    fn show_nav_button(&mut self, ui: &mut Ui, text: &str, page: Page) {
        let is_selected = self.page == page;
        let button = egui::Button::new(RichText::new(text).size(16.0).color(if is_selected {
            Color32::WHITE
        } else {
            Color32::LIGHT_GRAY
        }))
        .fill(if is_selected {
            Color32::DARK_BLUE
        } else {
            Color32::TRANSPARENT
        });

        if ui.add_sized([ui.available_width(), 40.0], button).clicked() {
            self.page = page;
        }
    }

    pub fn show_home(&mut self, ui: &mut Ui) {
        CollapsingHeader::new("Basic")
            .default_open(true)
            .show(ui, |ui| {
                styles::render_button(ui, "Migrate from your PDS to another PDS", || {
                    self.export_repo();
                });
                styles::render_button(ui, "Backup Repo", || {
                    self.export_repo();
                });
                styles::render_button(ui, "Backup Media", || {
                    self.export_repo();
                });
            });
        CollapsingHeader::new("Advanced")
            .default_open(false)
            .show(ui, |ui| {
                styles::render_button(ui, "Export Repo", || {
                    self.export_repo();
                });
                styles::render_button(ui, "Import Repo", || {
                    self.import_repo();
                });
                styles::render_button(ui, "Export Blobs", || {
                    self.export_blobs();
                });
                styles::render_button(ui, "Upload Blobs", || {
                    self.upload_blobs();
                });
                styles::render_button(ui, "Migrate Preferences", || {
                    self.migrate_preferences();
                });
                styles::render_button(ui, "Request Token", || {
                    self.request_token();
                });
                ui.horizontal(|ui| {
                    styles::render_button(ui, "Generate Recovery Key", || {
                        self.generate_recovery_key();
                    });
                    styles::render_input(
                        ui,
                        "Password",
                        &mut self.user_recovery_key_password,
                        true,
                        Some(""),
                    );
                });

                ui.horizontal(|ui| {
                    ui.horizontal(|ui| {
                        ui.vertical(|ui| {
                            ui.label("PLC Signing Token");
                            ui.text_edit_singleline(&mut self.plc_token);
                        });
                        ui.vertical(|ui| {
                            ui.label("User Recovery Key (optional)");
                            ui.text_edit_singleline(&mut self.user_recovery_key);
                        });
                    });
                });
                ui.horizontal(|ui| {
                    styles::render_button(ui, "Migrate with private key", || {
                        self.generate_recovery_key();
                    });
                });
                styles::render_button(ui, "Activate New Account", || {
                    self.activate_account();
                });
                styles::render_button(ui, "Deactivate Old Account", || {
                    self.deactivate_account();
                });
            });
        styles::render_button(ui, "Login to new PDS", || {
            self.deactivate_account();
        });
    }

    pub fn show_old_login(&mut self, ui: &mut Ui) {
        styles::render_subtitle(ui, "Current PDS Login");

        ui.vertical_centered(|ui| {
            styles::render_input(
                ui,
                "Current PDS Host",
                &mut self.old_pds_host,
                false,
                Some("https://bsky.social"),
            );
            styles::render_input(
                ui,
                "Handle",
                &mut self.username,
                false,
                Some("myaccount.bsky.social"),
            );
            styles::render_input(ui, "Password", &mut self.password, true, None);
            styles::render_button(ui, "Submit", || {
                self.old_session_login();
            });
        });
    }

    pub fn check_page_update(&mut self) {
        let res = self.page_rx.try_recv();
        if res.is_ok() {
            self.page = res.unwrap();
        }
    }

    pub fn check_for_errors(&mut self, _ctx: &egui::Context) {
        if let Ok(error) = self.error_rx.try_recv() {
            let error_window = ErrorWindow::new(error);
            self.error_windows.push(error_window);
        }
    }

    pub fn check_for_success(&mut self, _ctx: &egui::Context) {
        if let Ok(message) = self.success_rx.try_recv() {
            let success_window = SuccessWindow::new(message);
            self.success_windows.push(success_window);
        }
    }

    fn old_session_login(&mut self) {
        let old_pds_host = self.old_pds_host.to_string();
        let username = self.username.to_string();
        let password = self.password.to_string();
        let pds_session = self.pds_session.clone();
        let error_tx = self.error_tx.clone();
        let page_tx = self.page_tx.clone();
        tokio::spawn(async move {
            let bsky_agent = BskyAgent::builder().build().await.unwrap();
            match login_helper2(
                &bsky_agent,
                old_pds_host.as_str(),
                username.as_str(),
                password.as_str(),
            )
            .await
            {
                Ok(res) => {
                    let old_pds_token = res.access_jwt.clone();
                    let old_pds_refresh = res.refresh_jwt.clone();
                    let did = res.did.as_str().to_string();
                    let mut pds_session = pds_session.write().await;
                    pds_session.create_old_session(
                        did.as_str(),
                        old_pds_token.as_str(),
                        old_pds_refresh.as_str(),
                        old_pds_host.as_str(),
                    );
                    page_tx.send(Page::Home).unwrap();
                }
                Err(e) => {
                    error_tx.send(e).unwrap();
                }
            };
        });
    }

    fn export_repo(&mut self) {
        let pds_session_lock = self.pds_session.clone();
        let pds_session = pds_session_lock.blocking_read();
        let did = match pds_session.did().clone() {
            None => {
                panic!("No DID found");
            }
            Some(did) => did.to_string(),
        };

        let old_session_config = match &pds_session.old_session_config() {
            None => {
                panic!("No old session config found");
            }
            Some(config) => config,
        };
        let pds_host = old_session_config.host().to_string();
        let token = old_session_config.access_token().to_string();
        let success_tx = self.success_tx.clone();
        let error_tx = self.error_tx.clone();

        tokio::spawn(async move {
            success_tx
                .send("Exporting Repo Started".to_string())
                .unwrap();
            let request = ExportPDSRequest {
                pds_host,
                did,
                token,
            };
            match pdsmigration_common::export_pds_api(request).await {
                Ok(_) => {
                    // After parsing the response, notify the GUI thread of the increment value.
                    success_tx
                        .send("Export Repo Completed".to_string())
                        .unwrap();
                }
                Err(pds_error) => match pds_error {
                    PdsError::Login => {
                        error_tx.send(GuiError::InvalidLogin).unwrap();
                    }
                    PdsError::Runtime => {
                        error_tx.send(GuiError::Runtime).unwrap();
                    }
                    PdsError::AccountExport => {}
                    _ => {
                        error_tx.send(GuiError::Other).unwrap();
                    }
                },
            }
        });
    }

    fn import_repo(&mut self) {
        let pds_session_lock = self.pds_session.clone();
        let pds_session = pds_session_lock.blocking_read();
        let error_tx = self.error_tx.clone();
        let success_tx = self.success_tx.clone();

        let did = match pds_session.did().clone() {
            None => {
                error_tx.send(GuiError::Other).unwrap();
                return;
            }
            Some(did) => did.to_string(),
        };
        let new_session_config = match &pds_session.new_session_config() {
            None => {
                error_tx.send(GuiError::Other).unwrap();
                return;
            }
            Some(config) => config,
        };
        let pds_host = new_session_config.host().to_string();
        let token = new_session_config.access_token().to_string();

        tokio::spawn(async move {
            success_tx.send("Import Repo Started".to_string()).unwrap();
            let request = ImportPDSRequest {
                pds_host,
                did,
                token,
            };
            match pdsmigration_common::import_pds_api(request).await {
                Ok(_) => {
                    success_tx
                        .send("Import Repo Completed".to_string())
                        .unwrap();
                }
                Err(_) => {
                    error_tx.send(GuiError::Runtime).unwrap();
                }
            }
        });
    }

    fn export_blobs(&mut self) {
        let pds_session_lock = self.pds_session.clone();
        let pds_session = pds_session_lock.blocking_read();
        let error_tx = self.error_tx.clone();
        let success_tx = self.success_tx.clone();
        let did = match pds_session.did().clone() {
            None => {
                error_tx.send(GuiError::Other).unwrap();
                return;
            }
            Some(did) => did.to_string(),
        };
        let old_session_config = match &pds_session.old_session_config() {
            None => {
                error_tx.send(GuiError::Other).unwrap();
                return;
            }
            Some(config) => config,
        };
        let new_session_config = match &pds_session.new_session_config() {
            None => {
                error_tx.send(GuiError::Other).unwrap();
                return;
            }
            Some(config) => config,
        };
        let old_pds_host = old_session_config.host().to_string();
        let new_pds_host = new_session_config.host().to_string();
        let old_token = old_session_config.access_token().to_string();
        let new_token = new_session_config.access_token().to_string();

        tokio::spawn(async move {
            success_tx
                .send("Exporting Blobs Started".to_string())
                .unwrap();
            let request = ExportBlobsRequest {
                destination: new_pds_host,
                origin: old_pds_host,
                did,
                origin_token: old_token,
                destination_token: new_token,
            };
            match pdsmigration_common::export_blobs_api(request).await {
                Ok(_) => {
                    success_tx
                        .send("Export Blobs Completed".to_string())
                        .unwrap();
                }
                Err(pds_error) => match pds_error {
                    PdsError::Validation => {
                        error_tx.send(GuiError::Other).unwrap();
                    }
                    _ => {
                        error_tx.send(GuiError::Runtime).unwrap();
                    }
                },
            }
        });
    }

    fn upload_blobs(&mut self) {
        let pds_session_lock = self.pds_session.clone();
        let pds_session = pds_session_lock.blocking_read();
        let error_tx = self.error_tx.clone();
        let success_tx = self.success_tx.clone();
        let did = match pds_session.did().clone() {
            None => {
                error_tx.send(GuiError::Other).unwrap();
                return;
            }
            Some(did) => did.to_string(),
        };
        let new_session_config = match &pds_session.new_session_config() {
            None => {
                error_tx.send(GuiError::Other).unwrap();
                return;
            }
            Some(config) => config,
        };
        let pds_host = new_session_config.host().to_string();
        let token = new_session_config.access_token().to_string();

        tokio::spawn(async move {
            success_tx.send("Upload Blobs Started".to_string()).unwrap();
            let request = UploadBlobsRequest {
                pds_host,
                did,
                token,
            };
            match pdsmigration_common::upload_blobs_api(request).await {
                Ok(_) => {
                    success_tx
                        .send("Upload Blobs Completed".to_string())
                        .unwrap();
                }
                Err(_pds_error) => {
                    error_tx.send(GuiError::Runtime).unwrap();
                }
            }
        });
    }

    fn migrate_plc(&mut self) {
        let pds_session_lock = self.pds_session.clone();
        let pds_session = pds_session_lock.blocking_read();
        let error_tx = self.error_tx.clone();
        let success_tx = self.success_tx.clone();
        let did = match pds_session.did().clone() {
            None => {
                error_tx.send(GuiError::Other).unwrap();
                return;
            }
            Some(did) => did.to_string(),
        };
        let old_session_config = match &pds_session.old_session_config() {
            None => {
                error_tx.send(GuiError::Other).unwrap();
                return;
            }
            Some(config) => config,
        };
        let new_session_config = match &pds_session.new_session_config() {
            None => {
                error_tx.send(GuiError::Other).unwrap();
                return;
            }
            Some(config) => config,
        };
        let origin = old_session_config.host().to_string();
        let destination = new_session_config.host().to_string();
        let origin_token = old_session_config.access_token().to_string();
        let destination_token = new_session_config.access_token().to_string();
        let plc_signing_token = self.plc_token.clone();
        let user_recovery_key = match self.user_recovery_key.is_empty() {
            true => None,
            false => Some(self.user_recovery_key.clone()),
        };

        tokio::spawn(async move {
            success_tx.send("Migrate PLC Started".to_string()).unwrap();
            let request = MigratePlcRequest {
                destination,
                destination_token,
                origin,
                did,
                origin_token,
                plc_signing_token,
                user_recovery_key,
            };
            match pdsmigration_common::migrate_plc_api(request).await {
                Ok(_) => {
                    success_tx
                        .send("Migrate PLC Completed".to_string())
                        .unwrap();
                }
                Err(_pds_error) => {
                    error_tx.send(GuiError::Runtime).unwrap();
                }
            }
        });
    }

    fn migrate_preferences(&mut self) {
        let pds_session_lock = self.pds_session.clone();
        let pds_session = pds_session_lock.blocking_read();
        let error_tx = self.error_tx.clone();
        let success_tx = self.success_tx.clone();
        let did = match pds_session.did().clone() {
            None => {
                error_tx.send(GuiError::Other).unwrap();
                return;
            }
            Some(did) => did.to_string(),
        };
        let old_session_config = match &pds_session.old_session_config() {
            None => {
                error_tx.send(GuiError::Other).unwrap();
                return;
            }
            Some(config) => config,
        };
        let new_session_config = match &pds_session.new_session_config() {
            None => {
                error_tx.send(GuiError::Other).unwrap();
                return;
            }
            Some(config) => config,
        };
        let origin = old_session_config.host().to_string();
        let destination = new_session_config.host().to_string();
        let origin_token = old_session_config.access_token().to_string();
        let destination_token = new_session_config.access_token().to_string();

        tokio::spawn(async move {
            success_tx
                .send("Migrate Preferences Started".to_string())
                .unwrap();
            let request = MigratePreferencesRequest {
                destination,
                destination_token,
                origin,
                did,
                origin_token,
            };
            match pdsmigration_common::migrate_preferences_api(request).await {
                Ok(_) => {
                    success_tx
                        .send("Migrate Preferences Completed".to_string())
                        .unwrap();
                }
                Err(_pds_error) => {
                    error_tx.send(GuiError::Runtime).unwrap();
                }
            }
        });
    }

    fn request_token(&mut self) {
        let pds_session_lock = self.pds_session.clone();
        let pds_session = pds_session_lock.blocking_read();
        let error_tx = self.error_tx.clone();
        let success_tx = self.success_tx.clone();
        let did = match pds_session.did().clone() {
            None => {
                error_tx.send(GuiError::Other).unwrap();
                return;
            }
            Some(did) => did.to_string(),
        };
        let old_session_config = match &pds_session.old_session_config() {
            None => {
                error_tx.send(GuiError::Other).unwrap();
                return;
            }
            Some(config) => config,
        };
        let pds_host = old_session_config.host().to_string();
        let token = old_session_config.access_token().to_string();

        tokio::spawn(async move {
            success_tx
                .send("Request Token Started".to_string())
                .unwrap();
            let request = RequestTokenRequest {
                pds_host,
                did,
                token,
            };
            match pdsmigration_common::request_token_api(request).await {
                Ok(_) => {
                    success_tx
                        .send("Request Token Completed".to_string())
                        .unwrap();
                }
                Err(_pds_error) => {
                    error_tx.send(GuiError::Runtime).unwrap();
                }
            }
        });
    }

    fn multicodec_wrap(bytes: Vec<u8>) -> Vec<u8> {
        let mut buf = [0u8; 3];
        unsigned_varint::encode::u16(0xe7, &mut buf);
        let mut v: Vec<u8> = Vec::new();
        for b in &buf {
            v.push(*b);
            // varint uses first bit to indicate another byte follows, stop if not the case
            if *b <= 127 {
                break;
            }
        }
        v.extend(bytes);
        v
    }

    fn generate_recovery_key(&mut self) {
        let secp = Secp256k1::new();
        let (secret_key, public_key) = secp.generate_keypair(&mut rand::rng());
        let pk_compact = public_key.serialize();
        let pk_wrapped = Self::multicodec_wrap(pk_compact.to_vec());
        let pk_multibase = multibase::encode(Base58Btc, pk_wrapped.as_slice());
        let public_key_str = format!("did:key:{pk_multibase}");
        self.user_recovery_key = public_key_str;

        let sk_compact = secret_key.secret_bytes().to_vec();
        let sk_wrapped = Self::multicodec_wrap(sk_compact.to_vec());
        let sk_multibase = multibase::encode(Base58Btc, sk_wrapped.as_slice());
        let secret_key_str = format!("did:key:{sk_multibase}");

        let path = std::path::Path::new("RotationKey.zip");
        let file = std::fs::File::create(path).unwrap();

        let mut zip = ZipWriter::new(file);

        let options = SimpleFileOptions::default()
            .compression_method(zip::CompressionMethod::Stored)
            .with_aes_encryption(AesMode::Aes256, self.user_recovery_key_password.as_str());
        zip.start_file("RotationKey", options).unwrap();
        zip.write_all(secret_key_str.as_bytes()).unwrap();

        zip.finish().unwrap();
    }

    fn activate_account(&self) {
        let pds_session_lock = self.pds_session.clone();
        let pds_session = pds_session_lock.blocking_read();
        let error_tx = self.error_tx.clone();
        let success_tx = self.success_tx.clone();
        let did = match pds_session.did().clone() {
            None => {
                error_tx.send(GuiError::Other).unwrap();
                return;
            }
            Some(did) => did.to_string(),
        };
        let new_session_config = match &pds_session.new_session_config() {
            None => {
                error_tx.send(GuiError::Other).unwrap();
                return;
            }
            Some(config) => config,
        };
        let pds_host = new_session_config.host().to_string();
        let token = new_session_config.access_token().to_string();

        tokio::spawn(async move {
            success_tx
                .send("Activate Account Started".to_string())
                .unwrap();
            let request = ActivateAccountRequest {
                pds_host,
                did,
                token,
            };
            match pdsmigration_common::activate_account_api(request).await {
                Ok(_) => {
                    success_tx
                        .send("Activate Account Completed".to_string())
                        .unwrap();
                }
                Err(_pds_error) => {
                    error_tx.send(GuiError::Runtime).unwrap();
                }
            }
        });
    }

    fn deactivate_account(&self) {
        let pds_session_lock = self.pds_session.clone();
        let pds_session = pds_session_lock.blocking_read();
        let error_tx = self.error_tx.clone();
        let success_tx = self.success_tx.clone();
        let did = match pds_session.did().clone() {
            None => {
                error_tx.send(GuiError::Other).unwrap();
                return;
            }
            Some(did) => did.to_string(),
        };
        let old_session_config = match &pds_session.old_session_config() {
            None => {
                error_tx.send(GuiError::Other).unwrap();
                return;
            }
            Some(config) => config,
        };
        let pds_host = old_session_config.host().to_string();
        let token = old_session_config.access_token().to_string();

        tokio::spawn(async move {
            success_tx
                .send("Deactivate Account Started".to_string())
                .unwrap();
            let request = DeactivateAccountRequest {
                pds_host,
                did,
                token,
            };
            match pdsmigration_common::deactivate_account_api(request).await {
                Ok(_) => {
                    success_tx
                        .send("Deactivate Account Completed".to_string())
                        .unwrap();
                }
                Err(_pds_error) => {
                    error_tx.send(GuiError::Runtime).unwrap();
                }
            }
        });
    }

    pub fn show_new_login(&mut self, ui: &mut Ui) {
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

        // if self.login_rx.try_recv().is_ok() {
        //     self.new_session_login();
        // }
    }

    fn new_session_login(&mut self) {
        let new_pds_host = self.new_pds_host.to_string();
        let new_handle = self.new_handle.to_string();
        let new_password = self.new_password.to_string();
        let pds_session_lock = self.pds_session.clone();

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
                    let access_token = res.access_jwt.clone();
                    let refresh_token = res.refresh_jwt.clone();
                    let did = res.did.as_str().to_string();
                    let mut pds_session = pds_session_lock.blocking_write();
                    pds_session.create_new_session(
                        did.as_str(),
                        access_token.as_str(),
                        refresh_token.as_str(),
                        new_pds_host.as_str(),
                    );
                    // page_tx.send(Page::Home).unwrap();
                }
                Err(e) => {
                    // error_tx.send(e).unwrap();
                }
            };
        });
    }

    fn create_account(&mut self) {
        let pds_session_lock = self.pds_session.clone();
        let pds_session = pds_session_lock.blocking_write();
        let error_tx = self.error_tx.clone();
        let success_tx = self.success_tx.clone();
        let old_session_config = match &pds_session.old_session_config() {
            None => {
                error_tx.send(GuiError::Other).unwrap();
                return;
            }
            Some(session_config) => session_config.clone(),
        };
        let did = match pds_session.did().clone() {
            None => {
                error_tx.send(GuiError::Other).unwrap();
                return;
            }
            Some(did) => did.to_string(),
        };
        let email = self.new_email.clone();
        let new_pds_host = self.new_pds_host.clone();
        let aud = new_pds_host.replace("https://", "did:web:");

        let password = self.new_password.clone();
        let invite_code = self.invite_code.clone();
        let handle = self.new_handle.clone();

        tokio::spawn(async move {
            let service_auth_request = ServiceAuthRequest {
                pds_host: old_session_config.host().to_string(),
                aud,
                did: did.clone(),
                token: old_session_config.access_token().to_string(),
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
                    // login_tx.send(1).unwrap();
                }
                Err(_pds_error) => {
                    error_tx.send(GuiError::Runtime).unwrap();
                }
            }
        });
    }
}

impl Default for PdsMigrationApp {
    fn default() -> Self {
        let (error_tx, error_rx) = channel();
        let (success_tx, success_rx) = channel();
        let (page_tx, page_rx) = channel();
        Self {
            page: Page::OldLogin,
            error_windows: vec![],
            success_windows: vec![],
            pds_session: Arc::new(RwLock::new(PdsSession::new(None))),
            old_pds_host: "".to_string(),
            username: "".to_string(),
            password: "".to_string(),
            user_recovery_key_password: "".to_string(),
            plc_token: "".to_string(),
            user_recovery_key: "".to_string(),
            new_password: "".to_string(),
            new_pds_host: "".to_string(),
            new_email: "".to_string(),
            new_handle: "".to_string(),
            invite_code: "".to_string(),
            is_new_account: None,
            error_rx,
            error_tx,
            success_rx,
            success_tx,
            page_rx,
            page_tx,
        }
    }
}

impl eframe::App for PdsMigrationApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.check_page_update();
        self.check_for_errors(ctx);
        self.check_for_success(ctx);

        let mut new_error_windows = vec![];
        for error_window in &mut self.error_windows {
            if error_window.open {
                error_window.show(ctx);
                new_error_windows.push(error_window.clone());
            }
        }
        self.error_windows = new_error_windows;

        let mut new_success_windows = vec![];
        for success_window in &mut self.success_windows {
            if success_window.open {
                success_window.show(ctx);
                new_success_windows.push(success_window.clone());
            }
        }
        self.success_windows = new_success_windows;

        let lock = self.pds_session.clone();
        let pds_session = lock.blocking_read();
        let is_active_session = pds_session.did().is_some();

        // Left side panel for navigation buttons (arranged top-down)
        if is_active_session {
            egui::SidePanel::left("side_panel")
                .default_width(150.0)
                .show(ctx, |ui| {
                    ui.add_space(20.0);
                    ui.vertical_centered_justified(|ui| {
                        self.show_nav_button(ui, "Home", Page::Home);
                        ui.add_space(10.0);
                        self.show_nav_button(ui, "Signout Main PDS", Page::OldLogin);
                        ui.add_space(10.0);
                        self.show_nav_button(ui, "New Login", Page::NewLogin);
                    });

                    // Push a spacer at the bottom to demonstrate vertical spacing
                    ui.with_layout(Layout::bottom_up(Align::Center), |ui| {
                        ui.horizontal(|ui| {
                            ui.label("v1.0.0");
                            ui.add_space(10.0);
                        });
                        ui.horizontal(|ui| {
                            if ui.button("Light Mode").clicked() {
                                ctx.set_theme(egui::Theme::Light);
                            }
                            ui.add_space(10.0);
                            if ui.button("Dark Mode").clicked() {
                                ctx.set_theme(egui::Theme::Dark);
                            }
                        });
                    });
                });
        }

        // let styled_frame = styles::get_styled_frame();
        egui::CentralPanel::default().show(ctx, |ui| {
            styles::set_text_color(ui);

            match self.page {
                Page::Home => self.show_home(ui),
                Page::OldLogin => self.show_old_login(ui),
                Page::NewLogin => self.show_new_login(ui),
            }
        });
    }
}

#[derive(Clone)]
pub struct ErrorWindow {
    open: bool,
    gui_error: GuiError,
}

impl ErrorWindow {
    pub fn new(gui_error: GuiError) -> Self {
        Self {
            open: true,
            gui_error,
        }
    }

    pub fn show(&mut self, ctx: &egui::Context) -> Option<InnerResponse<Option<()>>> {
        Window::new(self.gui_error.to_string())
            .title_bar(false)
            .open(&mut self.open.clone())
            .vscroll(false)
            .resizable(false)
            .show(ctx, |ui| {
                ui.label(format!("{}", self.gui_error));
                let btn = ui.button("Ok");
                if btn.clicked() {
                    self.open = false;
                }
            })
    }
}

#[derive(Clone)]
pub struct SuccessWindow {
    open: bool,
    message: String,
}

impl SuccessWindow {
    pub fn new(message: String) -> Self {
        Self {
            open: true,
            message,
        }
    }

    pub fn show(&mut self, ctx: &egui::Context) -> Option<InnerResponse<Option<()>>> {
        Window::new(self.message.clone())
            .title_bar(false)
            .open(&mut self.open.clone())
            .vscroll(false)
            .resizable(false)
            .show(ctx, |ui| {
                ui.label(self.message.to_string());
                let btn = ui.button("Ok");
                if btn.clicked() {
                    self.open = false;
                }
            })
    }
}
