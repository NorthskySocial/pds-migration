use crate::agent::login_helper2;
use crate::error_window::ErrorWindow;
use crate::errors::GuiError;
use crate::session_config::PdsSession;
use crate::styles;
use crate::styles::WIDGET_SPACING_BASE;
use crate::success_window::SuccessWindow;
use bsky_sdk::BskyAgent;
use egui::{Align, Color32, Layout, RichText, ScrollArea, Theme, Ui};
use egui_tracing::EventCollector;
use multibase::Base::Base58Btc;
use pdsmigration_common::agent::login_helper;
use pdsmigration_common::errors::PdsError;
use pdsmigration_common::{
    ActivateAccountRequest, CreateAccountApiRequest, DeactivateAccountRequest,
    ExportAllBlobsRequest, ExportBlobsRequest, ExportPDSRequest, ImportPDSRequest,
    MigratePlcRequest, MigratePreferencesRequest, RequestTokenRequest, ServiceAuthRequest,
    UploadBlobsRequest,
};
use secp256k1::Secp256k1;
use std::io::Write;
use std::sync::Arc;
use tokio::sync::RwLock;
use zip::write::SimpleFileOptions;
use zip::{AesMode, ZipWriter};

#[derive(PartialEq, Clone)]
pub enum Page {
    Basic,
    Advanced,
    OldLogin,
    NewLogin,
    PdsMigrationFlow,
    AccountCreate,
}

#[derive(Clone)]
pub enum PdsMigrationStep {
    DoesAccountExist,
    CreateNewAccount,
    LoginToNewPds,
    ExportRepo,
    ImportRepo,
    ExportBlobs,
    ImportBlobs,
    MigratePreferences,
    CreateRecoveryKey,
    MigratePLC,
    ActiveAccounts,
    Success,
}

#[derive(Clone)]
pub struct PdsMigrationApp {
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
    pds_migration_step: Arc<RwLock<Option<PdsMigrationStep>>>,
    error: Arc<RwLock<Vec<GuiError>>>,
    page: Arc<RwLock<Page>>,
    collector: EventCollector,
    started_step: Arc<RwLock<bool>>,
}

impl PdsMigrationApp {
    pub fn new(_cc: &eframe::CreationContext<'_>, collector: EventCollector) -> Self {
        let mut app = Self::default();
        app.collector = collector;
        app
    }

    fn clear(&mut self) {
        let page_lock = self.page.clone();
        let mut page = page_lock.blocking_write();
        *page = Page::Basic;
        drop(page);
        self.error_windows = vec![];
        self.success_windows = vec![];
        self.old_pds_host = "".to_string();
        self.username = "".to_string();
        self.password = "".to_string();
        self.user_recovery_key_password = "".to_string();
        self.plc_token = "".to_string();
        self.user_recovery_key = "".to_string();
    }

    // Helper function to create consistent navigation buttons
    fn show_nav_button(&mut self, ui: &mut Ui, ctx: &egui::Context, text: &str, _page: Page) {
        let page_lock = self.page.clone();
        let page = page_lock.blocking_read().clone();
        let is_selected = page == _page;
        let theme = ctx.theme();

        let button = egui::Button::new(RichText::new(text).size(16.0).color(if is_selected {
            match theme {
                Theme::Dark => Color32::WHITE,
                Theme::Light => Color32::BLACK,
            }
        } else {
            match theme {
                Theme::Dark => Color32::LIGHT_GRAY,
                Theme::Light => Color32::DARK_GRAY,
            }
        }))
        .fill(if is_selected {
            match theme {
                Theme::Dark => Color32::DARK_BLUE,
                Theme::Light => Color32::LIGHT_BLUE,
            }
        } else {
            match theme {
                Theme::Dark => Color32::TRANSPARENT,
                Theme::Light => Color32::TRANSPARENT,
            }
        });

        drop(page);
        if ui.add_sized([ui.available_width(), 40.0], button).clicked() {
            let mut page = page_lock.blocking_write();
            *page = _page;
            drop(page)
        }
    }

    // Helper function to create consistent navigation buttons
    fn show_side_panel(&mut self, ctx: &egui::Context) {
        let lock = self.pds_session.clone();
        let pds_session = lock.blocking_read().clone();
        let is_active_session = pds_session.did().is_some();

        // Left side panel for navigation buttons (arranged top-down)
        if is_active_session {
            egui::SidePanel::left("side_panel")
                .default_width(100.0)
                .show(ctx, |ui| {
                    ui.add_space(20.0);
                    ui.vertical_centered_justified(|ui| {
                        self.show_nav_button(ui, ctx, "Basic", Page::Basic);
                        ui.add_space(10.0);
                        self.show_nav_button(ui, ctx, "Advanced", Page::Advanced);
                    });

                    // Push a spacer at the bottom to demonstrate vertical spacing
                    ui.with_layout(Layout::bottom_up(Align::Center), |ui| {
                        ui.horizontal(|ui| {
                            ui.label("v1.0.0");
                            ui.add_space(10.0);
                        });
                        ui.horizontal(|ui| {
                            if ui.button("Light Mode").clicked() {
                                ctx.set_theme(Theme::Light);
                            }
                            ui.add_space(10.0);
                            if ui.button("Dark Mode").clicked() {
                                ctx.set_theme(Theme::Dark);
                            }
                        });
                    });
                });
        }
    }

    fn show_bottom_panel(&mut self, ctx: &egui::Context) {
        egui::TopBottomPanel::bottom("bottom_panel").show(ctx, |ui| {
            ui.add(egui_tracing::Logs::new(self.collector.clone()));
        });
    }

    fn show_central_panel(&mut self, ctx: &egui::Context) {
        egui::CentralPanel::default().show(ctx, |ui| {
            styles::set_text_color(ui);

            let page_lock = self.page.clone();
            let page = page_lock.blocking_read().clone();
            match page {
                Page::OldLogin => self.show_old_login(ui, ctx),
                Page::NewLogin => self.show_new_login(ui, ctx),
                Page::AccountCreate => self.show_new_login(ui, ctx),
                Page::PdsMigrationFlow => {
                    self.show_migrate_pds_flow(ui, ctx);
                }
                Page::Basic => {
                    self.show_basic_home(ui, ctx);
                }
                Page::Advanced => {
                    self.show_advanced_home(ui, ctx);
                }
            }
        });
    }

    fn show_basic_home(&mut self, ui: &mut Ui, ctx: &egui::Context) {
        ScrollArea::both().show(ui, |ui| {
            styles::render_button(ui, ctx, "Migrate from your PDS to another PDS", || {
                let pds_migration_step = self.pds_migration_step.clone();

                let mut pds_migration_step = pds_migration_step.blocking_write();
                *pds_migration_step = Some(PdsMigrationStep::DoesAccountExist);

                let page_lock = self.page.clone();
                let mut page = page_lock.blocking_write();
                *page = Page::PdsMigrationFlow;
            });
            styles::render_button(ui, ctx, "Backup Repo", || {
                self.export_repo();
            });
            styles::render_button(ui, ctx, "Backup Media", || {
                self.export_all_blobs();
            });
        });
    }

    fn show_advanced_home(&mut self, ui: &mut Ui, ctx: &egui::Context) {
        ScrollArea::both().show(ui, |ui| {
            styles::render_button(ui, ctx, "Export Repo", || {
                self.export_repo();
            });
            styles::render_button(ui, ctx, "Import Repo", || {
                self.import_repo();
            });
            styles::render_button(ui, ctx, "Export Blobs", || {
                self.export_missing_blobs();
            });
            styles::render_button(ui, ctx, "Upload Blobs", || {
                self.upload_blobs();
            });
            styles::render_button(ui, ctx, "Migrate Preferences", || {
                self.migrate_preferences();
            });
            styles::render_button(ui, ctx, "Request Token", || {
                self.request_token();
            });
            ui.horizontal(|ui| {
                styles::render_button(ui, ctx, "Generate Recovery Key", || {
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
                styles::render_button(ui, ctx, "Migrate with private key", || {
                    self.generate_recovery_key();
                });
            });
            styles::render_button(ui, ctx, "Activate New Account", || {
                self.activate_account();
            });
            styles::render_button(ui, ctx, "Deactivate Old Account", || {
                self.deactivate_account();
            });
        });
    }

    fn show_old_login(&mut self, ui: &mut Ui, ctx: &egui::Context) {
        styles::render_subtitle(ui, ctx, "Current PDS Login");
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
            ui.add_space(WIDGET_SPACING_BASE);
            styles::render_button(ui, ctx, "Submit", || {
                self.old_session_login();
            });
        });
    }

    fn check_for_errors(&mut self, ctx: &egui::Context) {
        let error_lock = self.error.clone();
        let mut error = error_lock.blocking_write();
        if !error.is_empty() {
            for error in error.iter() {
                let error_window = ErrorWindow::new(error.clone());
                self.error_windows.push(error_window);
            }
        }
        error.clear();
        let mut new_error_windows = vec![];
        for error_window in &mut self.error_windows {
            if error_window.open() {
                error_window.show(ctx);
                new_error_windows.push(error_window.clone());
            }
        }
        self.error_windows = new_error_windows;
    }

    #[tracing::instrument(skip(self))]
    fn old_session_login(&mut self) {
        let old_pds_host = self.old_pds_host.to_string();
        let username = self.username.to_string();
        let password = self.password.to_string();
        let pds_session = self.pds_session.clone();
        let page_lock = self.page.clone();
        let error_lock = self.error.clone();
        tokio::spawn(async move {
            tracing::info!("Logging in to old PDS");
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
                    tracing::info!("Successfully logged in to old PDS");
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
                    let mut page = page_lock.write().await;
                    *page = Page::Basic;
                    drop(page)
                }
                Err(e) => {
                    tracing::error!("Error logging in to old PDS: {:?}", e);
                    let mut error = error_lock.write().await;
                    error.push(e);
                }
            };
        });
    }

    fn show_migrate_pds_flow(&mut self, ui: &mut Ui, ctx: &egui::Context) {
        let pds_migration_step_lock = self.pds_migration_step.clone();
        let pds_migration_step = pds_migration_step_lock.blocking_read().clone();
        match pds_migration_step {
            None => {
                styles::render_subtitle(ui, ctx, "Does your account exist on new PDS?");
                ui.vertical_centered(|ui| {
                    styles::render_button(ui, ctx, "Yes", || {
                        let mut pds_migration_step_write = pds_migration_step_lock.blocking_write();
                        *pds_migration_step_write = Some(PdsMigrationStep::LoginToNewPds)
                    });
                    styles::render_button(ui, ctx, "No", || {
                        let mut pds_migration_step_write = pds_migration_step_lock.blocking_write();
                        *pds_migration_step_write = Some(PdsMigrationStep::CreateNewAccount)
                    });
                });
            }
            Some(migration_step) => match migration_step {
                PdsMigrationStep::DoesAccountExist => {
                    styles::render_subtitle(ui, ctx, "Does your account exist on new PDS?");
                    ui.vertical_centered(|ui| {
                        styles::render_button(ui, ctx, "Yes", || {
                            let mut pds_migration_step_write =
                                pds_migration_step_lock.blocking_write();
                            *pds_migration_step_write = Some(PdsMigrationStep::LoginToNewPds)
                        });
                        styles::render_button(ui, ctx, "No", || {
                            let mut pds_migration_step_write =
                                pds_migration_step_lock.blocking_write();
                            *pds_migration_step_write = Some(PdsMigrationStep::CreateNewAccount)
                        });
                    });
                }
                PdsMigrationStep::CreateNewAccount => {
                    styles::render_subtitle(ui, ctx, "Create New PDS Account!");
                    ui.vertical_centered(|ui| {
                        ui.horizontal(|ui| {
                            styles::render_input(
                                ui,
                                "New PDS Host",
                                &mut self.new_pds_host,
                                false,
                                Some("https://northsky.social"),
                            );
                            styles::render_button(ui, ctx, "Update", || {
                                self.create_account();
                            });
                        });
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
                        styles::render_button(ui, ctx, "Submit", || {
                            self.create_account();
                        });
                    });
                }
                PdsMigrationStep::LoginToNewPds => {
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
                            let started_step_lock = self.started_step.clone();
                            let pds_migration_step_lock = pds_migration_step_lock.clone();
                            let mut started_step = started_step_lock.blocking_write();
                            *started_step = false;
                            let mut pds_migration_step = pds_migration_step_lock.blocking_write();
                            *pds_migration_step = Some(PdsMigrationStep::ExportRepo);
                        });
                    });
                }
                PdsMigrationStep::ExportRepo => {
                    styles::render_subtitle(ui, ctx, "Exporting Repo from old PDS");
                    let started = {
                        let started_step = self.started_step.blocking_read();
                        *started_step
                    };
                    if !started {
                        // Clone what we need for the async task
                        let started_step_lock = self.started_step.clone();
                        let pds_migration_step_lock = pds_migration_step_lock.clone();
                        let mut app_state = self.clone();
                        tokio::spawn(async move {
                            tracing::info!("Exporting repo from old PDS");
                            app_state.export_repo();
                            let mut started_step = started_step_lock.write().await;
                            *started_step = false;
                            let mut pds_migration_step = pds_migration_step_lock.write().await;
                            *pds_migration_step = Some(PdsMigrationStep::ImportRepo);
                        });
                    }
                }
                PdsMigrationStep::ImportRepo => {
                    styles::render_subtitle(ui, ctx, "Importing Repo to new PDS");
                    let started = {
                        let started_step = self.started_step.blocking_read();
                        *started_step
                    };
                    if !started {
                        // Clone what we need for the async task
                        let started_step_lock = self.started_step.clone();
                        let pds_migration_step_lock = pds_migration_step_lock.clone();
                        let mut app_state = self.clone();
                        tokio::spawn(async move {
                            tracing::info!("Importing repo to new PDS");
                            app_state.import_repo();
                            let mut started_step = started_step_lock.write().await;
                            *started_step = false;
                            let mut pds_migration_step = pds_migration_step_lock.write().await;
                            *pds_migration_step = Some(PdsMigrationStep::ExportBlobs);
                        });
                    }
                }
                PdsMigrationStep::ExportBlobs => {
                    styles::render_subtitle(ui, ctx, "Exportings blobs from old PDS");
                    let started = {
                        let started_step = self.started_step.blocking_read();
                        *started_step
                    };
                    if !started {
                        // Clone what we need for the async task
                        let started_step_lock = self.started_step.clone();
                        let pds_migration_step_lock = pds_migration_step_lock.clone();
                        let mut app_state = self.clone();
                        tokio::spawn(async move {
                            tracing::info!("Exporting blobs from old PDS");
                            app_state.export_missing_blobs();
                            let mut started_step = started_step_lock.write().await;
                            *started_step = false;
                            let mut pds_migration_step = pds_migration_step_lock.write().await;
                            *pds_migration_step = Some(PdsMigrationStep::ImportBlobs);
                        });
                    }
                }
                PdsMigrationStep::ImportBlobs => {
                    styles::render_subtitle(ui, ctx, "Importing Blobs from old PDS");
                    let started = {
                        let started_step = self.started_step.blocking_read();
                        *started_step
                    };
                    if !started {
                        // Clone what we need for the async task
                        let started_step_lock = self.started_step.clone();
                        let pds_migration_step_lock = pds_migration_step_lock.clone();
                        let mut app_state = self.clone();
                        tokio::spawn(async move {
                            tracing::info!("Importing blobs to new PDS");
                            app_state.upload_blobs();
                            let mut started_step = started_step_lock.write().await;
                            *started_step = false;
                            let mut pds_migration_step = pds_migration_step_lock.write().await;
                            *pds_migration_step = Some(PdsMigrationStep::MigratePreferences);
                        });
                    }
                }
                PdsMigrationStep::MigratePreferences => {
                    styles::render_subtitle(ui, ctx, "Migrating preferences");
                    let started = {
                        let started_step = self.started_step.blocking_read();
                        *started_step
                    };
                    if !started {
                        // Clone what we need for the async task
                        let started_step_lock = self.started_step.clone();
                        let pds_migration_step_lock = pds_migration_step_lock.clone();
                        let mut app_state = self.clone();
                        tokio::spawn(async move {
                            tracing::info!("Migrating preferences from old to new PDS");
                            app_state.migrate_preferences();
                            let mut started_step = started_step_lock.write().await;
                            *started_step = false;
                            let mut pds_migration_step = pds_migration_step_lock.write().await;
                            *pds_migration_step = Some(PdsMigrationStep::CreateRecoveryKey);
                        });
                    }
                }
                PdsMigrationStep::CreateRecoveryKey => {
                    //TODO
                    styles::render_subtitle(ui, ctx, "Create Recovery Key");
                    let started = {
                        let started_step = self.started_step.blocking_read();
                        *started_step
                    };
                    if !started {
                        // Clone what we need for the async task
                        let started_step_lock = self.started_step.clone();
                        let pds_migration_step_lock = pds_migration_step_lock.clone();
                        let mut app_state = self.clone();
                        tokio::spawn(async move {
                            tracing::info!("Importing blobs to new PDS");
                            app_state.upload_blobs();
                            let mut started_step = started_step_lock.write().await;
                            *started_step = false;
                            let mut pds_migration_step = pds_migration_step_lock.write().await;
                            *pds_migration_step = Some(PdsMigrationStep::MigratePLC);
                        });
                    }
                }
                PdsMigrationStep::MigratePLC => {
                    styles::render_subtitle(ui, ctx, "Updating PLC");
                    let started = {
                        let started_step = self.started_step.blocking_read();
                        *started_step
                    };
                    if !started {
                        // Clone what we need for the async task
                        let started_step_lock = self.started_step.clone();
                        let pds_migration_step_lock = pds_migration_step_lock.clone();
                        let mut app_state = self.clone();
                        tokio::spawn(async move {
                            tracing::info!("Updating PLC");
                            app_state.migrate_plc();
                            let mut started_step = started_step_lock.write().await;
                            *started_step = false;
                            let mut pds_migration_step = pds_migration_step_lock.write().await;
                            *pds_migration_step = Some(PdsMigrationStep::ActiveAccounts);
                        });
                    }
                }
                PdsMigrationStep::ActiveAccounts => {
                    styles::render_subtitle(
                        ui,
                        ctx,
                        "Deactivating old account, and activating new account",
                    );
                    let started_step_lock = self.started_step.clone();
                    let started_step_read = started_step_lock.blocking_read().clone();
                    let app_state = self.clone();
                    if !started_step_read {
                        let mut started_step_write = started_step_lock.blocking_write();
                        *started_step_write = true;
                        let started_step_lock = self.started_step.clone();
                        let pds_migration_step_lock = self.pds_migration_step.clone();
                        tokio::spawn(async move {
                            tracing::info!("Deactivating old account, and activating new account");
                            app_state.activate_account();
                            app_state.deactivate_account();
                            let mut started_step_write = started_step_lock.write().await;
                            *started_step_write = false;
                            let mut pds_migration_step_write =
                                pds_migration_step_lock.write().await;
                            *pds_migration_step_write = Some(PdsMigrationStep::ExportBlobs);
                        });
                    }
                }
                PdsMigrationStep::Success => {
                    styles::render_subtitle(
                        ui,
                        ctx,
                        "Congratulations, you are successfully migrated!",
                    );
                }
            },
        }
    }

    fn export_repo(&mut self) {
        let pds_session_lock = self.pds_session.clone();
        let pds_session = pds_session_lock.blocking_read().clone();
        let did = match pds_session.did().clone() {
            None => {
                tracing::error!("No DID found");
                panic!("No DID found");
            }
            Some(did) => did.to_string(),
        };

        let old_session_config = match &pds_session.old_session_config() {
            None => {
                tracing::error!("No old session config found");
                panic!("No old session config found");
            }
            Some(config) => config,
        };
        let pds_host = old_session_config.host().to_string();
        let token = old_session_config.access_token().to_string();
        let error_lock = self.error.clone();

        tokio::spawn(async move {
            tracing::info!("Exporting Repo started");
            let request = ExportPDSRequest {
                pds_host,
                did,
                token,
            };
            match pdsmigration_common::export_pds_api(request).await {
                Ok(_) => {
                    tracing::info!("Exporting Repo completed");
                }
                Err(pds_error) => {
                    tracing::error!("Error exporting repo: {:?}", pds_error);
                    let mut error = error_lock.write().await;
                    match pds_error {
                        PdsError::Login => {
                            error.push(GuiError::InvalidLogin);
                        }
                        PdsError::Runtime => {
                            error.push(GuiError::Runtime);
                        }
                        PdsError::AccountExport => {
                            error.push(GuiError::Other);
                        }
                        _ => {
                            error.push(GuiError::Other);
                        }
                    }
                }
            }
        });
    }

    fn import_repo(&mut self) {
        let pds_session_lock = self.pds_session.clone();
        let pds_session = pds_session_lock.blocking_read().clone();
        let error_lock = self.error.clone();

        let did = match pds_session.did().clone() {
            None => {
                tracing::error!("No DID found");
                let mut error = error_lock.blocking_write();
                error.push(GuiError::Other);
                return;
            }
            Some(did) => did.to_string(),
        };
        let new_session_config = match &pds_session.new_session_config() {
            None => {
                tracing::error!("No new session config found");
                let mut error = error_lock.blocking_write();
                error.push(GuiError::Other);
                return;
            }
            Some(config) => config,
        };
        let pds_host = new_session_config.host().to_string();
        let token = new_session_config.access_token().to_string();

        tokio::spawn(async move {
            tracing::info!("Importing Repo started");
            let request = ImportPDSRequest {
                pds_host,
                did,
                token,
            };
            match pdsmigration_common::import_pds_api(request).await {
                Ok(_) => {
                    tracing::info!("Importing Repo completed");
                }
                Err(pds_error) => {
                    tracing::error!("Error importing repo: {:?}", pds_error);
                    let mut error = error_lock.write().await;
                    error.push(GuiError::Runtime);
                }
            }
        });
    }

    fn export_missing_blobs(&mut self) {
        let pds_session_lock = self.pds_session.clone();
        let pds_session = pds_session_lock.blocking_read().clone();
        let error_lock = self.error.clone();
        let did = match pds_session.did().clone() {
            None => {
                tracing::error!("No DID found");
                let mut error = error_lock.blocking_write();
                error.push(GuiError::Other);
                return;
            }
            Some(did) => did.to_string(),
        };
        let old_session_config = match &pds_session.old_session_config() {
            None => {
                let mut error = error_lock.blocking_write();
                error.push(GuiError::Other);
                return;
            }
            Some(config) => config,
        };
        let new_session_config = match &pds_session.new_session_config() {
            None => {
                let mut error = error_lock.blocking_write();
                error.push(GuiError::Other);
                return;
            }
            Some(config) => config,
        };
        let old_pds_host = old_session_config.host().to_string();
        let new_pds_host = new_session_config.host().to_string();
        let old_token = old_session_config.access_token().to_string();
        let new_token = new_session_config.access_token().to_string();

        tokio::spawn(async move {
            tracing::info!("Exporting Missing Blobs started");
            let request = ExportBlobsRequest {
                destination: new_pds_host,
                origin: old_pds_host,
                did,
                origin_token: old_token,
                destination_token: new_token,
            };
            match pdsmigration_common::export_blobs_api(request).await {
                Ok(_) => {
                    tracing::info!("Exporting Missing Blobs completed");
                }
                Err(pds_error) => match pds_error {
                    PdsError::Validation => {
                        let mut error = error_lock.write().await;
                        error.push(GuiError::Other);
                    }
                    _ => {
                        let mut error = error_lock.write().await;
                        error.push(GuiError::Runtime);
                    }
                },
            }
        });
    }

    fn export_all_blobs(&mut self) {
        let pds_session_lock = self.pds_session.clone();
        let pds_session = pds_session_lock.blocking_read().clone();
        let error_lock = self.error.clone();
        let did = match pds_session.did().clone() {
            None => {
                let mut error = error_lock.blocking_write();
                error.push(GuiError::Other);
                return;
            }
            Some(did) => did.to_string(),
        };
        let old_session_config = match &pds_session.old_session_config() {
            None => {
                let mut error = error_lock.blocking_write();
                error.push(GuiError::Other);
                return;
            }
            Some(config) => config,
        };
        let old_pds_host = old_session_config.host().to_string();
        let old_token = old_session_config.access_token().to_string();

        tokio::spawn(async move {
            tracing::info!("Exporting All Blobs started");
            let request = ExportAllBlobsRequest {
                origin: old_pds_host,
                did,
                origin_token: old_token,
            };
            match pdsmigration_common::export_all_blobs_api(request).await {
                Ok(_) => {
                    tracing::info!("Exporting All Blobs completed");
                }
                Err(pds_error) => match pds_error {
                    PdsError::Validation => {
                        let mut error = error_lock.write().await;
                        error.push(GuiError::Other);
                    }
                    _ => {
                        let mut error = error_lock.write().await;
                        error.push(GuiError::Runtime);
                    }
                },
            }
        });
    }

    #[tracing::instrument(skip(self))]
    fn upload_blobs(&mut self) {
        let pds_session_lock = self.pds_session.clone();
        let pds_session = pds_session_lock.blocking_read().clone();
        let error_lock = self.error.clone();
        let did = match pds_session.did().clone() {
            None => {
                let mut error = error_lock.blocking_write();
                error.push(GuiError::Other);
                return;
            }
            Some(did) => did.to_string(),
        };
        let new_session_config = match &pds_session.new_session_config() {
            None => {
                let mut error = error_lock.blocking_write();
                error.push(GuiError::Other);
                return;
            }
            Some(config) => config,
        };
        let pds_host = new_session_config.host().to_string();
        let token = new_session_config.access_token().to_string();

        tokio::spawn(async move {
            tracing::info!("Uploading Blobs started");
            let request = UploadBlobsRequest {
                pds_host,
                did,
                token,
            };
            match pdsmigration_common::upload_blobs_api(request).await {
                Ok(_) => {
                    tracing::info!("Uploading Blobs completed");
                }
                Err(_pds_error) => {
                    tracing::error!("Error uploading blobs: {_pds_error}");
                    let mut error = error_lock.write().await;
                    error.push(GuiError::Runtime);
                }
            }
        });
    }

    #[tracing::instrument(skip(self))]
    fn migrate_plc(&mut self) {
        let pds_session_lock = self.pds_session.clone();
        let pds_session = pds_session_lock.blocking_read().clone();
        let error_lock = self.error.clone();
        let did = match pds_session.did().clone() {
            None => {
                let mut error = error_lock.blocking_write();
                error.push(GuiError::Other);
                return;
            }
            Some(did) => did.to_string(),
        };
        let old_session_config = match &pds_session.old_session_config() {
            None => {
                let mut error = error_lock.blocking_write();
                error.push(GuiError::Other);
                return;
            }
            Some(config) => config,
        };
        let new_session_config = match &pds_session.new_session_config() {
            None => {
                let mut error = error_lock.blocking_write();
                error.push(GuiError::Other);
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
            tracing::info!("Migrating PLC started");
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
                    tracing::info!("Migrating PLC completed");
                }
                Err(_pds_error) => {
                    tracing::error!("Error migrating PLC: {_pds_error}");
                    let mut error = error_lock.write().await;
                    error.push(GuiError::Runtime);
                }
            }
        });
    }

    #[tracing::instrument(skip(self))]
    fn migrate_preferences(&mut self) {
        let pds_session_lock = self.pds_session.clone();
        let pds_session = pds_session_lock.blocking_read().clone();
        let error_lock = self.error.clone();
        let did = match pds_session.did().clone() {
            None => {
                let mut error = error_lock.blocking_write();
                error.push(GuiError::Other);
                return;
            }
            Some(did) => did.to_string(),
        };
        let old_session_config = match &pds_session.old_session_config() {
            None => {
                let mut error = error_lock.blocking_write();
                error.push(GuiError::Other);
                return;
            }
            Some(config) => config,
        };
        let new_session_config = match &pds_session.new_session_config() {
            None => {
                let mut error = error_lock.blocking_write();
                error.push(GuiError::Other);
                return;
            }
            Some(config) => config,
        };
        let origin = old_session_config.host().to_string();
        let destination = new_session_config.host().to_string();
        let origin_token = old_session_config.access_token().to_string();
        let destination_token = new_session_config.access_token().to_string();

        tokio::spawn(async move {
            tracing::info!("Migrating Preferences started");
            let request = MigratePreferencesRequest {
                destination,
                destination_token,
                origin,
                did,
                origin_token,
            };
            match pdsmigration_common::migrate_preferences_api(request).await {
                Ok(_) => {
                    tracing::info!("Migrating Preferences completed");
                }
                Err(pds_error) => {
                    tracing::error!("Error migrating Preferences: {pds_error}");
                    let mut error = error_lock.write().await;
                    error.push(GuiError::Runtime);
                }
            }
        });
    }

    #[tracing::instrument(skip(self))]
    fn request_token(&mut self) {
        let pds_session_lock = self.pds_session.clone();
        let pds_session = pds_session_lock.blocking_read().clone();
        let error_lock = self.error.clone();
        let did = match pds_session.did().clone() {
            None => {
                let mut error = error_lock.blocking_write();
                error.push(GuiError::Other);
                return;
            }
            Some(did) => did.to_string(),
        };
        let old_session_config = match &pds_session.old_session_config() {
            None => {
                let mut error = error_lock.blocking_write();
                error.push(GuiError::Other);
                return;
            }
            Some(config) => config,
        };
        let pds_host = old_session_config.host().to_string();
        let token = old_session_config.access_token().to_string();

        tokio::spawn(async move {
            tracing::info!("Requesting Token started");
            let request = RequestTokenRequest {
                pds_host,
                did,
                token,
            };
            match pdsmigration_common::request_token_api(request).await {
                Ok(_) => {
                    tracing::info!("Requesting Token completed");
                }
                Err(pds_error) => {
                    tracing::error!("Error requesting token: {pds_error}");
                    let mut error = error_lock.write().await;
                    error.push(GuiError::Runtime);
                }
            }
        });
    }

    #[tracing::instrument(skip(self))]
    fn generate_recovery_key(&mut self) {
        let secp = Secp256k1::new();
        let (secret_key, public_key) = secp.generate_keypair(&mut rand::rng());
        let pk_compact = public_key.serialize();
        let pk_wrapped = multicodec_wrap(pk_compact.to_vec());
        let pk_multibase = multibase::encode(Base58Btc, pk_wrapped.as_slice());
        let public_key_str = format!("did:key:{pk_multibase}");
        self.user_recovery_key = public_key_str;

        let sk_compact = secret_key.secret_bytes().to_vec();
        let sk_wrapped = multicodec_wrap(sk_compact.to_vec());
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

    #[tracing::instrument(skip(self))]
    fn activate_account(&self) {
        let pds_session_lock = self.pds_session.clone();
        let pds_session = pds_session_lock.blocking_read().clone();
        let error_lock = self.error.clone();
        let did = match pds_session.did().clone() {
            None => {
                let mut error = error_lock.blocking_write();
                error.push(GuiError::Other);
                return;
            }
            Some(did) => did.to_string(),
        };
        let new_session_config = match &pds_session.new_session_config() {
            None => {
                let mut error = error_lock.blocking_write();
                error.push(GuiError::Other);
                return;
            }
            Some(config) => config,
        };
        let pds_host = new_session_config.host().to_string();
        let token = new_session_config.access_token().to_string();

        tokio::spawn(async move {
            tracing::info!("Activating Account started");
            let request = ActivateAccountRequest {
                pds_host,
                did,
                token,
            };
            match pdsmigration_common::activate_account_api(request).await {
                Ok(_) => {
                    tracing::info!("Activating Account completed");
                }
                Err(pds_error) => {
                    tracing::error!("Error activating account: {pds_error}");
                    let mut error = error_lock.write().await;
                    error.push(GuiError::Runtime);
                }
            }
        });
    }

    #[tracing::instrument(skip(self))]
    fn deactivate_account(&self) {
        let pds_session_lock = self.pds_session.clone();
        let pds_session = pds_session_lock.blocking_read().clone();
        let error_lock = self.error.clone();
        let did = match pds_session.did().clone() {
            None => {
                let mut error = error_lock.blocking_write();
                error.push(GuiError::Other);
                return;
            }
            Some(did) => did.to_string(),
        };
        let old_session_config = match &pds_session.old_session_config() {
            None => {
                let mut error = error_lock.blocking_write();
                error.push(GuiError::Other);
                return;
            }
            Some(config) => config,
        };
        let pds_host = old_session_config.host().to_string();
        let token = old_session_config.access_token().to_string();

        tokio::spawn(async move {
            tracing::info!("Deactivating Account started");
            let request = DeactivateAccountRequest {
                pds_host,
                did,
                token,
            };
            match pdsmigration_common::deactivate_account_api(request).await {
                Ok(_) => {
                    tracing::info!("Deactivating Account completed");
                }
                Err(pds_error) => {
                    tracing::error!("Error deactivating account: {pds_error}");
                    let mut error = error_lock.write().await;
                    error.push(GuiError::Runtime);
                }
            }
        });
    }

    fn show_new_login(&mut self, ui: &mut Ui, ctx: &egui::Context) {
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

    fn show_new_account_create(&mut self, ui: &mut Ui, ctx: &egui::Context) {
        styles::render_subtitle(ui, ctx, "Create New PDS Account!");
        ui.vertical_centered(|ui| {
            styles::render_input(
                ui,
                "New PDS Host",
                &mut self.new_pds_host,
                false,
                Some("https://northsky.social"),
            );
            styles::render_button(ui, ctx, "Submit", || {
                self.create_account();
            });
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
            styles::render_button(ui, ctx, "Submit", || {
                self.create_account();
            });
        });
    }

    #[tracing::instrument(skip(self))]
    fn show_choose_pds(&mut self) {
        let new_pds_host = self.new_pds_host.to_string();
        let new_handle = self.new_handle.to_string();
        let new_password = self.new_password.to_string();
        let pds_session_lock = self.pds_session.clone();
        let error_lock = self.error.clone();

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
                    tracing::info!("Login successful");
                    let access_token = res.access_jwt.clone();
                    let refresh_token = res.refresh_jwt.clone();
                    let did = res.did.as_str().to_string();
                    let mut pds_session = pds_session_lock.write().await;
                    pds_session.create_new_session(
                        did.as_str(),
                        access_token.as_str(),
                        refresh_token.as_str(),
                        new_pds_host.as_str(),
                    );
                }
                Err(e) => {
                    tracing::error!("Error logging in: {e}");
                    let mut error = error_lock.write().await;
                    error.push(GuiError::Runtime);
                }
            };
        });
    }

    #[tracing::instrument(skip(self))]
    fn new_session_login(&mut self) {
        let new_pds_host = self.new_pds_host.to_string();
        let new_handle = self.new_handle.to_string();
        let new_password = self.new_password.to_string();
        let pds_session_lock = self.pds_session.clone();
        let error_lock = self.error.clone();

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
                    tracing::info!("Login successful");
                    let access_token = res.access_jwt.clone();
                    let refresh_token = res.refresh_jwt.clone();
                    let did = res.did.as_str().to_string();
                    let mut pds_session = pds_session_lock.write().await;
                    pds_session.create_new_session(
                        did.as_str(),
                        access_token.as_str(),
                        refresh_token.as_str(),
                        new_pds_host.as_str(),
                    );
                }
                Err(e) => {
                    tracing::error!("Error logging in: {e}");
                    let mut error = error_lock.write().await;
                    error.push(GuiError::Other);
                }
            };
        });
    }

    #[tracing::instrument(skip(self))]
    fn create_account(&mut self) {
        let pds_session_lock = self.pds_session.clone();
        let pds_session = pds_session_lock.blocking_write();
        let error_lock = self.error.clone();

        let old_session_config = match &pds_session.old_session_config() {
            None => {
                let mut error = error_lock.blocking_write();
                error.push(GuiError::Other);
                return;
            }
            Some(session_config) => session_config.clone(),
        };
        let did = match pds_session.did().clone() {
            None => {
                let mut error = error_lock.blocking_write();
                error.push(GuiError::Other);
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
            tracing::info!("Creating Account started");
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
                        let mut error = error_lock.write().await;
                        error.push(GuiError::Runtime);
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
                    tracing::info!("Creating Account completed");
                    // login_tx.send(1).unwrap();
                }
                Err(pds_error) => {
                    tracing::error!("Error creating account: {pds_error}");
                    let mut error = error_lock.write().await;
                    error.push(GuiError::Runtime);
                }
            }
        });
    }
}

impl Default for PdsMigrationApp {
    fn default() -> Self {
        Self {
            page: Arc::new(RwLock::new(Page::OldLogin)),
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
            pds_migration_step: Arc::new(Default::default()),
            collector: Default::default(),
            started_step: Arc::new(Default::default()),
            error: Arc::new(RwLock::new(Default::default())),
        }
    }
}

impl eframe::App for PdsMigrationApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.check_for_errors(ctx);
        self.show_side_panel(ctx);
        self.show_central_panel(ctx);
        self.show_bottom_panel(ctx);
    }
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
