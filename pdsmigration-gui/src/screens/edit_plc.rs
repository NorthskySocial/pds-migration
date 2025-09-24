// use crate::errors::GuiError;
// use crate::screens::Screen;
// use crate::session::session_config::PdsSession;
// use crate::{styles, ScreenType};
// use egui::{Context, Ui};
// use egui_file_dialog::FileDialog;
// use std::path::PathBuf;
// use std::sync::Arc;
// use tokio::sync::RwLock;
//
// pub struct EditPlc {
//     pds_session: Arc<RwLock<PdsSession>>,
//     error: Arc<RwLock<Vec<GuiError>>>,
//     logged_in: bool,
//     rotation_secret_key: String,
//     page: Arc<RwLock<ScreenType>>,
//     repo_file_dialog: FileDialog,
//     picked_repo_file: Option<PathBuf>,
//     did: String,
// }
//
// impl EditPlc {
//     pub fn new(
//         pds_session: Arc<RwLock<PdsSession>>,
//         error: Arc<RwLock<Vec<GuiError>>>,
//         page: Arc<RwLock<ScreenType>>,
//     ) -> Self {
//         let logged_in = { pds_session.blocking_read().old_session_config().is_some() };
//         Self {
//             pds_session,
//             error,
//             logged_in,
//             rotation_secret_key: "".to_string(),
//             page,
//             repo_file_dialog: Default::default(),
//             picked_repo_file: None,
//             did: "".to_string(),
//         }
//     }
//
//     pub fn show_logged_in(&mut self, ui: &mut Ui, ctx: &Context) {
//         // ui.vertical_centered(|ui| {
//         //     ui.horizontal(|ui| {
//         //         if self.pds_selected {
//         //             styles::render_input_disabled(
//         //                 ui,
//         //                 "New PDS Host",
//         //                 &mut self.new_pds_host,
//         //                 false,
//         //                 Some("https://northsky.social"),
//         //             );
//         //             styles::render_button(ui, ctx, "Edit", || self.pds_selected = false);
//         //         } else {
//         //             styles::render_input(
//         //                 ui,
//         //                 "New PDS Host",
//         //                 &mut self.new_pds_host,
//         //                 false,
//         //                 Some("https://northsky.social"),
//         //             );
//         //             styles::render_button(ui, ctx, "Update", || {
//         //                 self.pds_selected = true;
//         //                 // self.update_pds();
//         //             });
//         //         }
//         //     });
//         //     if self.pds_selected {
//         //         styles::render_input(ui, "Email", &mut self.new_email, false, None);
//         //         styles::render_input(
//         //             ui,
//         //             "Handle",
//         //             &mut self.new_handle,
//         //             false,
//         //             Some("user.northsky.social"),
//         //         );
//         //         styles::render_input(ui, "Password", &mut self.new_password, true, None);
//         //         ui.horizontal_wrapped(|ui| {
//         //             ui.spacing_mut().item_spacing.x = 0.0;
//         //             ui.label("By creating an account you agree to the ");
//         //
//         //             if !terms_of_service.is_empty() {
//         //                 ui.hyperlink_to("Terms of Service", terms_of_service);
//         //                 if !privacy_policy.is_empty() {
//         //                     ui.label(" and ");
//         //                     ui.hyperlink_to("Privacy Policy", privacy_policy);
//         //                     ui.label(".");
//         //                 } else {
//         //                     ui.label(".");
//         //                 }
//         //             } else {
//         //                 // ui.hyperlink_to("Privacy Policy", privacy_policy);
//         //                 ui.label(".");
//         //             }
//         //         });
//         //         styles::render_button(ui, ctx, "Update", || {
//         //             //todo
//         //         });
//         //     }
//         // });
//     }
//
//     pub fn show_logged_out(&mut self, ui: &mut Ui, ctx: &Context) {
//         // ui.vertical_centered(|ui| {
//         //     ui.horizontal(|ui| {
//         //         todo!();
//         //         todo!();
//         //     })
//         // });
//     }
// }
//
// impl Screen for EditPlc {
//     fn ui(&mut self, ui: &mut Ui, ctx: &egui::Context) {
//         styles::render_subtitle(ui, ctx, "Edit PLC");
//
//         if self.logged_in {
//             self.show_logged_in(ui, ctx)
//         } else {
//             self.show_logged_out(ui, ctx)
//         }
//     }
//
//     fn name(&self) -> ScreenType {
//         ScreenType::EditPLC
//     }
// }
//
// #[cfg(test)]
// mod tests {}
