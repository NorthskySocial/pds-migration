#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod agent;

// hide console window on Windows in release
use crate::agent::login_helper;
use bsky_sdk::api::agent::atp_agent::AtpSession;
use bsky_sdk::BskyAgent;
use eframe::egui;
use egui::Window;
use pdsmigration_common::{
    CreateAccountApiRequest, ExportBlobsRequest, ExportPDSRequest, ImportPDSRequest,
    MigratePlcRequest, MigratePreferencesRequest, RequestTokenRequest, ServiceAuthRequest,
    UploadBlobsRequest,
};
use std::sync::mpsc::{Receiver, Sender};
use std::time::Duration;
use tokio::runtime::Runtime;

enum Page {
    Home,
    OldLogin,
    NewLogin,
    CreateAccount,
}

fn main() -> eframe::Result {
    env_logger::init(); // Log to stderr (if you run with `RUST_LOG=debug`).
    let rt = Runtime::new().expect("Unable to create Runtime");

    // Enter the runtime so that `tokio::spawn` is available immediately.
    let _enter = rt.enter();

    // Execute the runtime in its own thread.
    // The future doesn't have to do anything. In this example, it just sleeps forever.
    std::thread::spawn(move || {
        rt.block_on(async {
            loop {
                tokio::time::sleep(Duration::from_secs(3600)).await;
            }
        })
    });

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default(),
        ..Default::default()
    };

    eframe::run_native(
        " PDS Migration Tool",
        options,
        Box::new(|_cc| Ok(Box::<PdsMigrationApp>::default())),
    )
}

struct PdsMigrationApp {
    did: Option<String>,
    old_pds_token: Option<String>,
    new_pds_token: Option<String>,
    success_open: bool,
    error_open: bool,

    username: String,
    password: String,
    page: Page,

    plc_token: String,
    user_recovery_key: String,

    old_pds_host: String,
    new_pds_host: String,
    new_handle: String,

    invite_code: String,
    // old_pds_host: String,
    new_email: String,
    new_password: String,

    // Sender/Receiver for async notifications.
    tx: Sender<u32>,
    rx: Receiver<u32>,

    // Sender/Receiver for login attempts to old pds
    login_tx: Sender<AtpSession>,
    login_rx: Receiver<AtpSession>,

    // Sender/Receiver for login attempts to old pds
    new_login_tx: Sender<AtpSession>,
    new_login_rx: Receiver<AtpSession>,
}

impl Default for PdsMigrationApp {
    fn default() -> Self {
        let (tx, rx) = std::sync::mpsc::channel();
        let (login_tx, login_rx) = std::sync::mpsc::channel();
        let (new_login_tx, new_login_rx) = std::sync::mpsc::channel();
        Self {
            did: None,
            old_pds_token: None,
            new_pds_token: None,
            success_open: false,
            error_open: false,
            username: "".to_string(),
            password: "".to_string(),
            page: Page::Home,
            plc_token: "".to_string(),
            user_recovery_key: "".to_string(),
            old_pds_host: "https://bsky.social".to_string(),
            new_pds_host: "https://pds.ripperoni.com".to_string(),
            new_handle: "".to_string(),
            invite_code: "".to_string(),
            new_email: "".to_string(),
            new_password: "".to_string(),
            tx,
            rx,
            login_tx,
            login_rx,
            new_login_tx,
            new_login_rx,
        }
    }
}

impl eframe::App for PdsMigrationApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        Window::new("Success")
            .open(&mut self.success_open.clone())
            .vscroll(false)
            .resizable(false)
            .show(ctx, |ui| {
                let btn = ui.button("Ok");
                if btn.clicked() {
                    self.success_open = false;
                }
            });
        Window::new("Error")
            .open(&mut self.error_open.clone())
            .vscroll(false)
            .resizable(false)
            .show(ctx, |ui| {
                let btn = ui.button("Ok");
                if btn.clicked() {
                    self.error_open = false;
                }
            });

        egui::CentralPanel::default().show(ctx, |ui| {
            let res = self.login_rx.try_recv();
            if res.is_ok() {
                let session = res.unwrap();
                self.did = Some(session.did.as_str().to_string());
                self.old_pds_token = Some(session.access_jwt.clone());
                self.page = Page::Home;
            }

            let res = self.new_login_rx.try_recv();
            if res.is_ok() {
                let session = res.unwrap();
                self.did = Some(session.did.as_str().to_string());
                self.new_pds_token = Some(session.access_jwt.clone());
                self.page = Page::Home;
            }

            let res = self.rx.try_recv();
            if let Ok(res) = res {
                if res == 1 {
                    self.success_open = true;
                } else if res == 2 {
                    self.error_open = true;
                }
            }

            match &mut self.page {
                Page::Home => {
                    if self.old_pds_token.is_some() {
                        if self.new_pds_token.is_none() {
                            ui.horizontal(|ui| {
                                if ui.button("Login to New PDS").clicked() {
                                    self.page = Page::NewLogin;
                                }
                            });
                            ui.horizontal(|ui| {
                                if ui.button("Create Account").clicked() {
                                    self.page = Page::CreateAccount;
                                }
                            });
                        } else {
                            ui.horizontal(|ui| {
                                if ui.button("Export Repo").clicked() {
                                    export_repo(self, self.tx.clone(), ctx.clone());
                                }
                            });
                            ui.separator();
                            ui.horizontal(|ui| {
                                if ui.button("Import Repo").clicked() {
                                    import_repo(self, self.tx.clone(), ctx.clone());
                                }
                            });
                            ui.separator();
                            ui.horizontal(|ui| {
                                if ui.button("Export Blobs").clicked() {
                                    export_blobs(self, self.tx.clone(), ctx.clone());
                                }
                            });
                            ui.separator();
                            ui.horizontal(|ui| {
                                if ui.button("Upload Blobs").clicked() {
                                    upload_blobs(self, self.tx.clone(), ctx.clone());
                                }
                            });
                            ui.separator();
                            ui.horizontal(|ui| {
                                if ui.button("Migrate Preferences").clicked() {
                                    migrate_preferences(self, self.tx.clone(), ctx.clone());
                                }
                            });
                            ui.separator();
                            ui.horizontal(|ui| {
                                if ui.button("Request Token").clicked() {
                                    request_token(self, self.tx.clone(), ctx.clone());
                                }
                            });
                            ui.separator();
                            ui.horizontal(|ui| {
                                ui.horizontal(|ui| {
                                    ui.vertical(|ui| {
                                        ui.label("PLC Signing Token");
                                        ui.text_edit_singleline(&mut self.plc_token);
                                    });
                                    ui.vertical(|ui| {
                                        ui.label("User Recovery Key (optional)");
                                        ui.text_edit_singleline(&mut self.plc_token);
                                    });
                                    ui.vertical(|ui| {
                                        if ui.button("Migrate PLC").clicked() {
                                            migrate_plc(self, self.tx.clone(), ctx.clone());
                                        }
                                    });
                                });
                            });
                            ui.separator();
                            ui.horizontal(|ui| {
                                if ui.button("Activate Account").clicked() {
                                    // activate_account(self, self.tx.clone(), ctx.clone());
                                }
                            });
                            ui.separator();
                            ui.horizontal(|ui| {
                                if ui.button("Deactivate Account").clicked() {
                                    // deactivate_account(self, self.tx.clone(), ctx.clone());
                                }
                            });
                            ui.separator();
                            ui.horizontal(|ui| {
                                if self.old_pds_token.is_some() {
                                    ui.label("Old PDS Status: Logged In");
                                }
                                if self.new_pds_token.is_some() {
                                    ui.label("New PDS Status: Logged In");
                                }
                            });
                        }
                    } else {
                        ui.horizontal(|ui| {
                            if ui.button(" Login to current PDS").clicked() {
                                self.page = Page::OldLogin;
                            }
                        });
                    }
                }
                Page::OldLogin => {
                    ui.heading("Old PDS Login");
                    ui.horizontal(|ui| {
                        ui.label("Old PDS: ");
                        ui.text_edit_singleline(&mut self.old_pds_host);
                    });
                    ui.horizontal(|ui| {
                        ui.label("Username: ");
                        ui.text_edit_singleline(&mut self.username);
                    });
                    ui.horizontal(|ui| {
                        ui.label("Password: ");
                        ui.add(egui::TextEdit::singleline(&mut self.password).password(true));
                    });
                    if ui.button("Submit").clicked() {
                        old_session_login(
                            self,
                            self.tx.clone(),
                            self.login_tx.clone(),
                            ctx.clone(),
                        );
                    }
                }
                Page::NewLogin => {
                    ui.heading("New PDS Login");
                    ui.horizontal(|ui| {
                        ui.label("New PDS: ");
                        ui.text_edit_singleline(&mut self.new_pds_host);
                    });
                    ui.horizontal(|ui| {
                        ui.label("Username: ");
                        ui.text_edit_singleline(&mut self.new_handle);
                    });
                    ui.horizontal(|ui| {
                        ui.label("Password: ");
                        ui.add(egui::TextEdit::singleline(&mut self.new_password).password(true));
                    });
                    if ui.button("Submit").clicked() {
                        new_session_login(
                            self,
                            self.tx.clone(),
                            self.new_login_tx.clone(),
                            ctx.clone(),
                        );
                    }
                }
                Page::CreateAccount => {
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
                        create_account(self, self.tx.clone(), ctx.clone());
                    }
                }
            }
        });
    }
}

fn migrate_plc(app: &PdsMigrationApp, tx: Sender<u32>, ctx: egui::Context) {
    let did = app.did.clone().unwrap();
    let old_pds_host = app.old_pds_host.to_string();
    let new_pds_host = app.new_pds_host.to_string();
    let old_token = app.old_pds_token.clone().unwrap();
    let new_token = app.new_pds_token.clone().unwrap();
    let plc_signing_token = app.plc_token.clone();
    let user_recovery_key = match app.user_recovery_key.is_empty() {
        true => None,
        false => Some(app.user_recovery_key.clone()),
    };

    tokio::spawn(async move {
        let request = MigratePlcRequest {
            new_pds_host,
            new_token,
            old_pds_host,
            did,
            old_token,
            plc_signing_token,
            user_recovery_key,
        };
        match pdsmigration_common::migrate_plc_api(request).await {
            Ok(_) => {
                let _ = tx.send(1);
            }
            Err(_) => {
                let _ = tx.send(2);
            }
        }
        ctx.request_repaint();
    });
}

fn request_token(app: &PdsMigrationApp, tx: Sender<u32>, ctx: egui::Context) {
    let did = app.did.clone().unwrap();
    let pds_host = app.old_pds_host.clone();
    let token = app.old_pds_token.clone().unwrap();

    tokio::spawn(async move {
        let request = RequestTokenRequest {
            pds_host,
            did,
            token,
        };
        match pdsmigration_common::request_token_api(request).await {
            Ok(_) => {
                let _ = tx.send(1);
            }
            Err(_) => {
                let _ = tx.send(2);
            }
        }

        ctx.request_repaint();
    });
}

fn migrate_preferences(app: &PdsMigrationApp, tx: Sender<u32>, ctx: egui::Context) {
    let did = app.did.clone().unwrap();
    let old_pds_host = app.old_pds_host.to_string();
    let new_pds_host = app.new_pds_host.to_string();
    let old_token = app.old_pds_token.clone().unwrap();
    let new_token = app.new_pds_token.clone().unwrap();

    tokio::spawn(async move {
        let request = MigratePreferencesRequest {
            new_pds_host,
            new_token,
            old_pds_host,
            did,
            old_token,
        };
        match pdsmigration_common::migrate_preferences_api(request).await {
            Ok(_) => {
                // After parsing the response, notify the GUI thread of the increment value.
                let _ = tx.send(1);
                ctx.request_repaint();
            }
            Err(_) => {
                // After parsing the response, notify the GUI thread of the increment value.
                let _ = tx.send(2);
                ctx.request_repaint();
            }
        }
    });
}

fn export_blobs(app: &PdsMigrationApp, tx: Sender<u32>, ctx: egui::Context) {
    let did = app.did.clone().unwrap();
    let old_pds_host = app.old_pds_host.to_string();
    let new_pds_host = app.new_pds_host.to_string();
    let old_token = app.old_pds_token.clone().unwrap();
    let new_token = app.new_pds_token.clone().unwrap();

    tokio::spawn(async move {
        let request = ExportBlobsRequest {
            new_pds_host,
            old_pds_host,
            did,
            old_token,
            new_token,
        };
        match pdsmigration_common::export_blobs_api(request).await {
            Ok(_) => {
                // After parsing the response, notify the GUI thread of the increment value.
                let _ = tx.send(1);
                ctx.request_repaint();
            }
            Err(_) => {
                // After parsing the response, notify the GUI thread of the increment value.
                let _ = tx.send(2);
                ctx.request_repaint();
            }
        }
    });
}

fn upload_blobs(app: &PdsMigrationApp, tx: Sender<u32>, ctx: egui::Context) {
    let did = app.did.clone().unwrap();
    let pds_host = app.new_pds_host.to_string();
    let token = app.new_pds_token.clone().unwrap();

    tokio::spawn(async move {
        let request = UploadBlobsRequest {
            pds_host,
            did,
            token,
        };
        match pdsmigration_common::upload_blobs_api(request).await {
            Ok(_) => {
                // After parsing the response, notify the GUI thread of the increment value.
                let _ = tx.send(1);
                ctx.request_repaint();
            }
            Err(_) => {
                // After parsing the response, notify the GUI thread of the increment value.
                let _ = tx.send(2);
                ctx.request_repaint();
            }
        }
    });
}

fn export_repo(app: &PdsMigrationApp, tx: Sender<u32>, ctx: egui::Context) {
    let did = app.did.clone().unwrap();
    let pds_host = app.old_pds_host.to_string();
    let token = app.old_pds_token.clone().unwrap();

    tokio::spawn(async move {
        let request = ExportPDSRequest {
            pds_host,
            did,
            token,
        };
        match pdsmigration_common::export_pds_api(request).await {
            Ok(_) => {
                // After parsing the response, notify the GUI thread of the increment value.
                let _ = tx.send(1);
                ctx.request_repaint();
            }
            Err(_) => {
                // After parsing the response, notify the GUI thread of the increment value.
                let _ = tx.send(2);
                ctx.request_repaint();
            }
        }
    });
}

fn import_repo(app: &PdsMigrationApp, tx: Sender<u32>, ctx: egui::Context) {
    let did = app.did.clone().unwrap();
    let pds_host = app.new_pds_host.to_string();
    let token = app.new_pds_token.clone().unwrap();

    tokio::spawn(async move {
        let request = ImportPDSRequest {
            pds_host,
            did,
            token,
        };
        match pdsmigration_common::import_pds_api(request).await {
            Ok(_) => {
                // After parsing the response, notify the GUI thread of the increment value.
                let _ = tx.send(1);
                ctx.request_repaint();
            }
            Err(_) => {
                // After parsing the response, notify the GUI thread of the increment value.
                let _ = tx.send(2);
                ctx.request_repaint();
            }
        }
    });
}

fn create_account(app: &PdsMigrationApp, tx: Sender<u32>, ctx: egui::Context) {
    let did = app.did.clone().unwrap();
    let token = app.old_pds_token.clone().unwrap().to_string();
    let email = app.new_email.clone();
    let pds_host = app.old_pds_host.clone();
    let new_pds_host = app.new_pds_host.clone();
    let aud = new_pds_host.replace("https://", "did:web:");

    let password = app.new_password.clone();
    let invite_code = app.invite_code.clone();
    let handle = app.new_handle.clone();

    tokio::spawn(async move {
        let service_auth_request = ServiceAuthRequest {
            pds_host: pds_host.clone(),
            aud,
            did: did.clone(),
            token: token.clone(),
        };
        let token = match pdsmigration_common::get_service_auth_api(service_auth_request).await {
            Ok(res) => res,
            Err(_) => {
                let _ = tx.send(2);
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
                let _ = tx.send(1);
                ctx.request_repaint();
            }
            Err(_) => {
                let _ = tx.send(2);
                ctx.request_repaint();
            }
        }
    });
}

fn old_session_login(
    app: &PdsMigrationApp,
    tx: Sender<u32>,
    login_tx: Sender<AtpSession>,
    ctx: egui::Context,
) {
    let old_pds_host = app.old_pds_host.to_string();
    let username = app.username.to_string();
    let password = app.password.to_string();
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
                let _ = tx.send(10);
                login_tx.send(res).unwrap();
            }
            Err(_) => {
                let _ = tx.send(2);
                panic!("");
            }
        };
        // After parsing the response, notify the GUI thread of the increment value.
        ctx.request_repaint();
    });
}

fn new_session_login(
    app: &PdsMigrationApp,
    tx: Sender<u32>,
    login_tx: Sender<AtpSession>,
    ctx: egui::Context,
) {
    let new_pds_host = app.new_pds_host.to_string();
    let new_handle = app.new_handle.to_string();
    let new_password = app.new_password.to_string();
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
                let _ = tx.send(10);
                login_tx.send(res).unwrap();
            }
            Err(_) => {
                let _ = tx.send(2);
                panic!("");
            }
        };

        // After parsing the response, notify the GUI thread of the increment value.
        ctx.request_repaint();
    });
}
