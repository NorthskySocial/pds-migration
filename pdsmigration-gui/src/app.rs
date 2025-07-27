use crate::error_window::ErrorWindow;
use crate::errors::GuiError;
use crate::screens::basic_home::BasicHome;
use crate::screens::old_login::OldLogin;
use crate::screens::Screen;
use crate::session::session_config::PdsSession;
use crate::{screens, styles, ScreenType};
use egui::{Align, Color32, Layout, RichText, Theme, Ui};
use egui_tracing::EventCollector;
use std::sync::Arc;
use tokio::sync::RwLock;

pub struct PdsMigrationApp {
    current_screen: Box<dyn Screen>,
    error_windows: Vec<ErrorWindow>,
    pds_session: Arc<RwLock<PdsSession>>,
    pds_migration_step: Arc<RwLock<bool>>,
    error: Arc<RwLock<Vec<GuiError>>>,
    page: Arc<RwLock<ScreenType>>,
    collector: EventCollector,
}

impl PdsMigrationApp {
    pub fn new(_cc: &eframe::CreationContext<'_>, collector: EventCollector) -> Self {
        Self {
            collector,
            ..Default::default()
        }
    }

    // Helper function to create consistent navigation buttons
    fn show_nav_button(&mut self, ui: &mut Ui, ctx: &egui::Context, text: &str, _page: ScreenType) {
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
                        self.show_nav_button(ui, ctx, "Basic", ScreenType::Basic);
                        // ui.add_space(10.0);
                        // self.show_nav_button(ui, ctx, "Advanced (WIP)", ScreenType::Advanced);
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
                        ui.add(
                            egui::Image::new(egui::include_image!(
                                "../assets/Northsky-Icon_Color.png"
                            ))
                            .shrink_to_fit(),
                        )
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
            self.current_screen.ui(ui, ctx);
        });
    }

    fn update_current_screen(&mut self) {
        // Get the current page
        let page = self.page.blocking_read().clone();

        if self.current_screen.name() == page {
            return;
        }

        // Based on the page, create a new screen
        let new_screen: Box<dyn Screen> = match page {
            ScreenType::Basic => Box::new(BasicHome::new(
                self.pds_session.clone(),
                self.error.clone(),
                self.page.clone(),
                self.pds_migration_step.clone(),
            )),
            ScreenType::OldLogin => Box::new(OldLogin::new(
                self.pds_session.clone(),
                self.error.clone(),
                self.page.clone(),
            )),
            ScreenType::NewLogin => Box::new(screens::new_login::NewLogin::new(
                self.pds_session.clone(),
                self.error.clone(),
                self.page.clone(),
                self.pds_migration_step.clone(),
            )),
            ScreenType::AccountCreate => Box::new(screens::create_account::CreateAccount::new(
                self.pds_session.clone(),
                self.error.clone(),
                self.page.clone(),
                self.pds_migration_step.clone(),
            )),
            ScreenType::MigratePLC => Box::new(screens::migrate_plc::MigratePLC::new(
                self.pds_session.clone(),
                self.error.clone(),
                self.page.clone(),
                self.pds_migration_step.clone(),
            )),
            ScreenType::Success => Box::new(screens::success::Success::new(
                self.page.clone(),
                self.pds_migration_step.clone(),
            )),
            ScreenType::ExportBlobs => Box::new(screens::export_blobs::ExportBlobs::new(
                self.pds_session.clone(),
                self.error.clone(),
                self.page.clone(),
                self.pds_migration_step.clone(),
            )),
            ScreenType::ImportBlobs => Box::new(screens::import_blobs::ImportBlobs::new(
                self.pds_session.clone(),
                self.error.clone(),
                self.page.clone(),
                self.pds_migration_step.clone(),
            )),
            ScreenType::MigratePreferences => {
                Box::new(screens::migrate_preferences::MigratePreferences::new(
                    self.pds_session.clone(),
                    self.error.clone(),
                    self.page.clone(),
                    self.pds_migration_step.clone(),
                ))
            }
            ScreenType::ActiveAccounts => Box::new(
                screens::deactivate_and_activate::DeactivateAndActivate::new(
                    self.pds_session.clone(),
                    self.error.clone(),
                    self.page.clone(),
                    self.pds_migration_step.clone(),
                ),
            ),
            ScreenType::DoesAccountExist => Box::new(screens::does_pds_exist::DoesPdsExist::new(
                self.page.clone(),
            )),
            ScreenType::CreateNewAccount => Box::new(screens::create_account::CreateAccount::new(
                self.pds_session.clone(),
                self.error.clone(),
                self.page.clone(),
                self.pds_migration_step.clone(),
            )),
            ScreenType::ExportRepo => Box::new(screens::export_repo::ExportRepo::new(
                self.pds_session.clone(),
                self.error.clone(),
                self.page.clone(),
            )),
            ScreenType::ImportRepo => Box::new(screens::import_repo::ImportRepo::new(
                self.pds_session.clone(),
                self.error.clone(),
                self.page.clone(),
            )),
        };

        // Reassign the current_screen
        self.current_screen = new_screen;
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
}

impl Default for PdsMigrationApp {
    fn default() -> Self {
        let pds_session = Arc::new(RwLock::new(PdsSession::new(None)));
        let page = Arc::new(RwLock::new(ScreenType::OldLogin));
        let error = Arc::new(RwLock::new(Default::default()));
        let pds_migration_step = Arc::new(RwLock::new(Default::default()));
        let current_screen = Box::new(BasicHome::new(
            pds_session.clone(),
            error.clone(),
            page.clone(),
            pds_migration_step.clone(),
        ));
        Self {
            current_screen,
            page,
            error_windows: vec![],
            pds_session: Arc::new(RwLock::new(PdsSession::new(None))),
            pds_migration_step,
            collector: Default::default(),
            error,
        }
    }
}

impl eframe::App for PdsMigrationApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.check_for_errors(ctx);
        self.update_current_screen();
        self.show_side_panel(ctx);
        self.show_central_panel(ctx);
        self.show_bottom_panel(ctx);
    }
}
