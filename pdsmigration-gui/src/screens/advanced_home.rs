// use crate::errors::GuiError;
// use crate::screens::Screen;
// use crate::session::session_config::PdsSession;
// use crate::{
//     activate_account, deactivate_account, export_missing_blobs, export_repo, generate_recovery_key,
//     import_repo, migrate_preferences, request_token, styles, upload_blobs,
//     ScreenType,
// };
// use egui::{ScrollArea, Ui};
// use std::sync::Arc;
// use tokio::sync::RwLock;
//
// #[derive(Clone)]
// pub struct AdvancedHome {
//     new_pds_host: String,
//     new_handle: String,
//     new_password: String,
//     pds_session: Arc<RwLock<PdsSession>>,
//     error: Arc<RwLock<Vec<GuiError>>>,
//     page: Arc<RwLock<ScreenType>>,
//     pds_migration_step: Arc<RwLock<bool>>,
// }
//
// impl AdvancedHome {
//     pub fn new(
//         pds_session: Arc<RwLock<PdsSession>>,
//         error: Arc<RwLock<Vec<GuiError>>>,
//         page: Arc<RwLock<ScreenType>>,
//         pds_migration_step: Arc<RwLock<bool>>,
//     ) -> Self {
//         Self {
//             new_pds_host: "".to_string(),
//             new_handle: "".to_string(),
//             new_password: "".to_string(),
//             pds_session,
//             error,
//             page,
//             pds_migration_step,
//         }
//     }
// }
// impl Screen for AdvancedHome {
//     fn ui(&mut self, ui: &mut Ui, ctx: &egui::Context) {
//         ScrollArea::both().show(ui, |ui| {
//             styles::render_button(ui, ctx, "Export Repo", || {
//                 export_repo();
//             });
//             styles::render_button(ui, ctx, "Import Repo", || {
//                 import_repo();
//             });
//             styles::render_button(ui, ctx, "Export Blobs", || {
//                 export_missing_blobs();
//             });
//             styles::render_button(ui, ctx, "Upload Blobs", || {
//                 upload_blobs();
//             });
//             styles::render_button(ui, ctx, "Migrate Preferences", || {
//                 migrate_preferences();
//             });
//             styles::render_button(ui, ctx, "Request Token", || {
//                 request_token();
//             });
//             ui.horizontal(|ui| {
//                 styles::render_button(ui, ctx, "Generate Recovery Key", || {
//                     generate_recovery_key();
//                 });
//                 styles::render_input(
//                     ui,
//                     "Password",
//                     &mut self.user_recovery_key_password,
//                     true,
//                     Some(""),
//                 );
//             });
//
//             ui.horizontal(|ui| {
//                 ui.horizontal(|ui| {
//                     ui.vertical(|ui| {
//                         ui.label("PLC Signing Token");
//                         ui.text_edit_singleline(&mut self.plc_token);
//                     });
//                     ui.vertical(|ui| {
//                         ui.label("User Recovery Key (optional)");
//                         ui.text_edit_singleline(&mut self.user_recovery_key);
//                     });
//                 });
//             });
//             ui.horizontal(|ui| {
//                 styles::render_button(ui, ctx, "Migrate with private key", || {
//                     generate_recovery_key();
//                 });
//             });
//             styles::render_button(ui, ctx, "Activate New Account", || {
//                 activate_account();
//             });
//             styles::render_button(ui, ctx, "Deactivate Old Account", || {
//                 deactivate_account();
//             });
//         });
//     }
//
//     fn name(&self) -> ScreenType {
//         ScreenType::Advanced
//     }
// }
