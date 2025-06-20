use crate::errors::GuiError;
use crate::{styles, Page};
use egui::Ui;
use multibase::Base::Base58Btc;
use pdsmigration_common::errors::PdsError;
use pdsmigration_common::{
    ActivateAccountRequest, DeactivateAccountRequest, ExportBlobsRequest, ExportPDSRequest,
    ImportPDSRequest, MigratePlcRequest, MigratePreferencesRequest, RequestTokenRequest,
    UploadBlobsRequest,
};
use secp256k1::Secp256k1;
use std::io::Write;
use std::sync::mpsc::Sender;
use zip::write::SimpleFileOptions;
use zip::{AesMode, ZipWriter};

pub struct HomePage {
    old_pds_token: String,
    new_pds_token: String,
    did: String,
    new_pds_host: String,
    old_pds_host: String,
    plc_token: String,
    user_recovery_key: String,
    user_recovery_key_password: String,
    error_tx: Sender<GuiError>,
    success_tx: Sender<String>,
}

impl HomePage {
    pub fn new(
        _page_tx: Sender<Page>,
        error_tx: Sender<GuiError>,
        success_tx: Sender<String>,
        old_pds_token: String,
        new_pds_token: String,
        old_pds_host: String,
        new_pds_host: String,
        did: String,
    ) -> Self {
        Self {
            old_pds_token,
            new_pds_token,
            did,
            new_pds_host,
            old_pds_host,
            plc_token: "".to_string(),
            error_tx,
            success_tx,
            user_recovery_key: "".to_string(),
            user_recovery_key_password: "".to_string(),
        }
    }

    pub fn show(&mut self, ui: &mut Ui) {
        ui.vertical_centered(|ui| {
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
                    ui.vertical(|ui| {
                        if ui.button("Migrate PLC").clicked() {
                            self.migrate_plc();
                        }
                    });
                });
            });
            styles::render_button(ui, "Activate New Account", || {
                self.activate_account();
            });
            styles::render_button(ui, "Deactivate Old Account", || {
                self.deactivate_account();
            });
        });
    }

    fn export_repo(&mut self) {
        let did = self.did.clone();
        let pds_host = self.old_pds_host.clone();
        let token = self.old_pds_token.clone();
        let error_tx = self.error_tx.clone();
        let success_tx = self.success_tx.clone();

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
        let did = self.did.clone();
        let pds_host = self.new_pds_host.to_string();
        let token = self.new_pds_token.clone();
        let error_tx = self.error_tx.clone();
        let success_tx = self.success_tx.clone();

        tokio::spawn(async move {
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
        let did = self.did.clone();
        let old_pds_host = self.old_pds_host.clone();
        let new_pds_host = self.new_pds_host.clone();
        let old_token = self.old_pds_token.clone();
        let new_token = self.new_pds_token.clone();
        let error_tx = self.error_tx.clone();
        let success_tx = self.success_tx.clone();

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
                Err(_pds_error) => {
                    error_tx.send(GuiError::Runtime).unwrap();
                }
            }
        });
    }

    fn upload_blobs(&mut self) {
        let did = self.did.clone();
        let pds_host = self.new_pds_host.clone();
        let token = self.new_pds_token.clone();
        let error_tx = self.error_tx.clone();
        let success_tx = self.success_tx.clone();

        tokio::spawn(async move {
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
        let did = self.did.clone();
        let origin = self.old_pds_host.clone();
        let destination = self.new_pds_host.clone();
        let origin_token = self.old_pds_token.clone();
        let destination_token = self.new_pds_token.clone();
        let plc_signing_token = self.plc_token.clone();
        let user_recovery_key = match self.user_recovery_key.is_empty() {
            true => None,
            false => Some(self.user_recovery_key.clone()),
        };
        let error_tx = self.error_tx.clone();
        let success_tx = self.success_tx.clone();

        tokio::spawn(async move {
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
        let did = self.did.clone();
        let origin = self.old_pds_host.clone();
        let destination = self.new_pds_host.clone();
        let origin_token = self.old_pds_token.clone();
        let destination_token = self.new_pds_token.clone();
        let error_tx = self.error_tx.clone();
        let success_tx = self.success_tx.clone();

        tokio::spawn(async move {
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
        let did = self.did.clone();
        let pds_host = self.old_pds_host.clone();
        let token = self.old_pds_token.clone();
        let error_tx = self.error_tx.clone();
        let success_tx = self.success_tx.clone();

        tokio::spawn(async move {
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
        zip.write_all(&secret_key_str.as_bytes()[..]).unwrap();

        zip.finish().unwrap();
    }

    fn activate_account(&self) {
        let did = self.did.clone();
        let pds_host = self.new_pds_host.clone();
        let token = self.new_pds_token.clone();
        let error_tx = self.error_tx.clone();
        let success_tx = self.success_tx.clone();

        tokio::spawn(async move {
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
        let did = self.did.clone();
        let pds_host = self.old_pds_host.clone();
        let token = self.old_pds_token.clone();
        let error_tx = self.error_tx.clone();
        let success_tx = self.success_tx.clone();

        tokio::spawn(async move {
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
}
