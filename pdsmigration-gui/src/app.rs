use crate::errors::GuiError;
use crate::existing_pds_page::ExistingPdsPage;
use crate::home_page::HomePage;
use crate::new_pds_page::NewPdsPage;
use crate::styles;
use egui::{InnerResponse, Window};
use std::sync::mpsc::Receiver;

pub enum Page {
    Home(HomePage),
    OldLogin(ExistingPdsPage),
    NewLogin(NewPdsPage),
}

// #[derive(Deserialize, Serialize)]
pub struct PdsMigrationApp {
    page: Page,
    page_rx: Receiver<Page>,
    error_rx: Receiver<GuiError>,
    error_windows: Vec<ErrorWindow>,
    success_rx: Receiver<String>,
    success_windows: Vec<SuccessWindow>,
}

impl PdsMigrationApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // This is also where you can customize the look and feel of egui using
        // `cc.egui_ctx.set_visuals` and `cc.egui_ctx.set_fonts`.

        // Load previous app state (if any).
        // Note that you must enable the `persistence` feature for this to work.
        // if let Some(storage) = cc.storage {
        //     return eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default();
        // }

        Default::default()
    }
}
impl Default for PdsMigrationApp {
    fn default() -> Self {
        let (page_tx, page_rx) = std::sync::mpsc::channel();
        let (error_tx, error_rx) = std::sync::mpsc::channel();
        let (success_tx, success_rx) = std::sync::mpsc::channel();

        Self {
            page: Page::OldLogin(ExistingPdsPage::new(page_tx, error_tx, success_tx)),
            page_rx,
            error_rx,
            error_windows: vec![],
            success_rx,
            success_windows: vec![],
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

        let styled_frame = styles::get_styled_frame();
        egui::CentralPanel::default()
            .frame(styled_frame)
            .show(ctx, |ui| {
                styles::set_text_color(ui);

                match &mut self.page {
                    Page::Home(home_page) => {
                        home_page.show(ui);
                    }
                    Page::OldLogin(existing_pds_page) => {
                        existing_pds_page.show(ui);
                    }
                    Page::NewLogin(new_pds_page) => {
                        new_pds_page.show(ui);
                    }
                }
            });
    }
}

impl PdsMigrationApp {
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
