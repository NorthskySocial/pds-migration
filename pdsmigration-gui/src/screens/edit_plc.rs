use crate::errors::GuiError;
use crate::screens::Screen;
use crate::session::session_config::PdsSession;
use crate::{styles, ScreenType};
use egui::Ui;
use egui_file_dialog::FileDialog;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;

pub struct EditPlc {
    new_pds_host: String,
    new_handle: String,
    new_password: String,
    pds_session: Option<PdsSession>,
    error: Arc<RwLock<Vec<GuiError>>>,
    pds_selected: bool,
    new_email: String,
    rotation_secret_key: String,
    page: Arc<RwLock<ScreenType>>,
    repo_file_dialog: FileDialog,
    picked_repo_file: Option<PathBuf>,
    did: String,
}

impl EditPlc {
    pub fn new(error: Arc<RwLock<Vec<GuiError>>>, page: Arc<RwLock<ScreenType>>) -> Self {
        Self {
            new_pds_host: "".to_string(),
            new_handle: "".to_string(),
            new_password: "".to_string(),
            pds_session: None,
            error,
            pds_selected: false,
            new_email: "".to_string(),
            rotation_secret_key: "".to_string(),
            page,
            repo_file_dialog: Default::default(),
            picked_repo_file: None,
            did: "".to_string(),
        }
    }
}

impl Screen for EditPlc {
    fn ui(&mut self, ui: &mut Ui, ctx: &egui::Context) {
        styles::render_subtitle(ui, ctx, "Edit PLC");
        ui.vertical_centered(|ui| {
            ui.horizontal(|ui| {
                if self.pds_selected {
                    styles::render_input_disabled(
                        ui,
                        "New PDS Host",
                        &mut self.new_pds_host,
                        false,
                        Some("https://northsky.social"),
                    );
                    styles::render_button(ui, ctx, "Edit", || self.pds_selected = false);
                } else {
                    styles::render_input(
                        ui,
                        "New PDS Host",
                        &mut self.new_pds_host,
                        false,
                        Some("https://northsky.social"),
                    );
                    styles::render_button(ui, ctx, "Update", || {
                        self.pds_selected = true;
                        // self.update_pds();
                    });
                }
            });
            if self.pds_selected {
                styles::render_input(ui, "Email", &mut self.new_email, false, None);
                styles::render_input(
                    ui,
                    "Handle",
                    &mut self.new_handle,
                    false,
                    Some("user.northsky.social"),
                );
                styles::render_input(ui, "Password", &mut self.new_password, true, None);
                ui.horizontal_wrapped(|ui| {
                    // ui.spacing_mut().item_spacing.x = 0.0;
                    // ui.label("By creating an account you agree to the ");
                    //
                    // if !terms_of_service.is_empty() {
                    //     ui.hyperlink_to("Terms of Service", terms_of_service);
                    //     if !privacy_policy.is_empty() {
                    //         ui.label(" and ");
                    //         ui.hyperlink_to("Privacy Policy", privacy_policy);
                    //         ui.label(".");
                    //     } else {
                    //         ui.label(".");
                    //     }
                    // } else {
                    //     // ui.hyperlink_to("Privacy Policy", privacy_policy);
                    //     ui.label(".");
                    // }
                });
                styles::render_button(ui, ctx, "Update", || {
                    //todo
                });
            }
        });
    }

    fn name(&self) -> ScreenType {
        ScreenType::MigrateWithoutPds
    }
}

#[cfg(test)]
mod tests {}
