#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod agent;

// hide console window on Windows in release
use crate::agent::login_helper;
use bsky_sdk::api::agent::atp_agent::AtpSession;
use bsky_sdk::BskyAgent;
use eframe::egui;
use egui::UiKind::Popup;
use egui::{PopupCloseBehavior, TextBuffer};
use pdsmigration_common::ServiceAuthRequest;
use std::sync::mpsc::{Receiver, Sender};
use std::time::Duration;
use tokio::runtime::Runtime;

const CHAR_LIMIT: i32 = 300;
const DIRECTORY_NAME: &str = "AdventureNodes";

enum Page {
    Home,
    Login(Login),
    ServiceAuth(ServiceAuth),
}

struct Login {
    old_pds_host: String,
    username: String,
    password: String,
}

struct ServiceAuth {
    old_pds_host: String,
    username: String,
    password: String,
    new_pds_host: String,
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
        " Adventurer Nodes",
        options,
        Box::new(|cc| Ok(Box::<PdsMigrationApp>::default())),
    )
}

struct PdsMigrationApp {
    did: Option<String>,
    old_pds_token: Option<String>,
    new_pds_token: Option<String>,

    page: Page,

    // Sender/Receiver for async notifications.
    tx: Sender<u32>,
    rx: Receiver<u32>,

    // Sender/Receiver for login attempts to old pds
    login_tx: Sender<AtpSession>,
    login_rx: Receiver<AtpSession>,

    // Silly app state.
    value: u32,
    count: u32,
}

impl PdsMigrationApp {
    fn clear(&mut self) {
        self.page = Page::Home;
    }
}

impl Default for PdsMigrationApp {
    fn default() -> Self {
        let (tx, rx) = std::sync::mpsc::channel();
        let (login_tx, login_rx) = std::sync::mpsc::channel();
        Self {
            did: None,
            old_pds_token: None,
            new_pds_token: None,
            page: Page::Home,
            tx,
            rx,
            login_tx,
            login_rx,
            value: 1,
            count: 0,
        }
    }
}

impl eframe::App for PdsMigrationApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // let response = ui.button("Open");
        //
        // Popup::menu(&response)
        //     .close_behavior(PopupCloseBehavior::IgnoreClicks)
        //     .show(|ui| {
        //         ui.set_min_width(310.0);
        //         ui.label("This popup will be open until you press the button again");
        //         ui.checkbox(&mut self.checkbox, "Checkbox");
        //     });
        egui::CentralPanel::default().show(ctx, |ui| {
            let res = self.login_rx.try_recv();
            if res.is_ok() {
                let session = res.unwrap();
                self.did = Some(session.did.as_str().to_string());
                self.old_pds_token = Some(session.access_jwt.clone());
                self.page = Page::Home;
            }

            match &mut self.page {
                Page::Home => {
                    if self.did.is_some() {
                        ui.horizontal(|ui| {
                            if ui.button("Service Auth").clicked() {
                                self.page = Page::ServiceAuth(ServiceAuth {
                                    old_pds_host: "https://bsky.social".to_string(),
                                    username: "".to_string(),
                                    password: "".to_string(),
                                    new_pds_host: "".to_string(),
                                });
                            }
                        });
                    } else {
                        ui.horizontal(|ui| {
                            if ui.button("Login").clicked() {
                                self.page = Page::Login(Login {
                                    old_pds_host: "https://bsky.social".to_string(),
                                    username: "".to_string(),
                                    password: "".to_string(),
                                });
                            }
                        });
                    }
                }
                Page::Login(ref mut login) => {
                    ui.heading("Login");
                    ui.horizontal(|ui| {
                        ui.label("Old PDS: ");
                        ui.text_edit_singleline(&mut login.old_pds_host);
                    });
                    ui.horizontal(|ui| {
                        ui.label("Username: ");
                        ui.text_edit_singleline(&mut login.username);
                    });
                    ui.horizontal(|ui| {
                        ui.label("Password: ");
                        ui.text_edit_singleline(&mut login.password);
                    });
                    if ui.button("Submit").clicked() {
                        session_login(login, self.tx.clone(), self.login_tx.clone(), ctx.clone());
                    }
                }
                Page::ServiceAuth(service_auth) => {
                    ui.heading("Get Service Auth");
                    ui.horizontal(|ui| {
                        ui.label("Old PDS Host: ");
                        ui.text_edit_singleline(&mut service_auth.old_pds_host);
                    });
                    ui.horizontal(|ui| {
                        ui.label("New PDS Host: ");
                        ui.text_edit_singleline(&mut service_auth.new_pds_host);
                    });
                    ui.horizontal(|ui| {
                        ui.label("Username: ");
                        ui.text_edit_singleline(&mut service_auth.username);
                    });
                    ui.horizontal(|ui| {
                        ui.label("Password: ");
                        ui.text_edit_singleline(&mut service_auth.password);
                    });
                    if ui.button("Submit").clicked() {
                        get_service_auth(self, self.tx.clone(), ctx.clone());
                    }
                }
            }
        });
    }
}

fn get_service_auth(service_auth_app: &PdsMigrationApp, tx: Sender<u32>, ctx: egui::Context) {
    tokio::spawn(async move {
        let service_auth_request = ServiceAuthRequest {
            pds_host: "".to_string(),
            aud: "".to_string(),
            did: "".to_string(),
            token: "".to_string(),
        };
        pdsmigration_common::get_service_auth_api(service_auth_request)
            .await
            .unwrap();

        // After parsing the response, notify the GUI thread of the increment value.
        let _ = tx.send(1);
        ctx.request_repaint();
    });
}

fn session_login(login: &Login, tx: Sender<u32>, login_tx: Sender<AtpSession>, ctx: egui::Context) {
    let old_pds_host = login.old_pds_host.to_string();
    let username = login.username.to_string();
    let password = login.password.to_string();
    tokio::spawn(async move {
        let bsky_agent = BskyAgent::builder().build().await.unwrap();
        let session = login_helper(
            &bsky_agent,
            old_pds_host.as_str(),
            username.as_str(),
            password.as_str(),
        )
        .await
        .unwrap();

        login_tx.send(session).unwrap();

        // After parsing the response, notify the GUI thread of the increment value.
        let _ = tx.send(1);
        ctx.request_repaint();
    });
}
