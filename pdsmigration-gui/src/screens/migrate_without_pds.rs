// use crate::errors::GuiError;
// use crate::screens::Screen;
// use crate::session::session_config::PdsSession;
// use crate::{
//     create_service_jwt, create_update_op, encode_did_key, fetch_tos_and_privacy_policy,
//     import_repo, migrate_preferences, styles, upload_blobs, ScreenType, ServiceJwtParams,
// };
// use bsky_sdk::api::agent::Configure;
// use bsky_sdk::api::types::string::Did;
// use bsky_sdk::BskyAgent;
// use egui::Ui;
// use egui_file_dialog::FileDialog;
// use pdsmigration_common::agent::{
//     create_account, get_plc_audit_log, get_recommended, send_plc_operation, PlcOpService,
//     PlcOperation,
// };
// use pdsmigration_common::CreateAccountRequest;
// use secp256k1::ffi::PublicKey;
// use secp256k1::{Keypair, Secp256k1, SecretKey};
// use std::path::PathBuf;
// use std::str::FromStr;
// use std::sync::Arc;
// use tokio::sync::RwLock;
//
// enum MigrationStep {
//     StepOne,
//     StepTwo,
// }
//
// pub struct MigrateWithoutPds {
//     new_pds_host: String,
//     new_handle: String,
//     new_password: String,
//     pds_session: Option<PdsSession>,
//     error: Arc<RwLock<Vec<GuiError>>>,
//     pds_selected: bool,
//     new_email: String,
//     _private_rotation_key: Option<SecretKey>,
//     public_rotation_key: Option<PublicKey>,
//     rotation_keypair: Option<Keypair>,
//     rotation_secret_key: String,
//     invite_code: String,
//     privacy_policy_lock: Arc<RwLock<Option<String>>>,
//     terms_of_service_lock: Arc<RwLock<Option<String>>>,
//     invite_code_required: Arc<RwLock<bool>>,
//     page: Arc<RwLock<ScreenType>>,
//     current_step: MigrationStep,
//     repo_file_dialog: FileDialog,
//     picked_repo_file: Option<PathBuf>,
//     service_jwt_token: Option<String>,
//     temporary_signing_keypair: Keypair,
//     did: String,
// }
//
// impl MigrateWithoutPds {
//     pub fn new(error: Arc<RwLock<Vec<GuiError>>>, page: Arc<RwLock<ScreenType>>) -> Self {
//         let secp = Secp256k1::new();
//         let keypair = Keypair::new(&secp, &mut rand::rng());
//         let temporary_signing_keypair = keypair;
//         Self {
//             new_pds_host: "".to_string(),
//             new_handle: "".to_string(),
//             new_password: "".to_string(),
//             pds_session: None,
//             error,
//             pds_selected: false,
//             new_email: "".to_string(),
//             _private_rotation_key: None,
//             public_rotation_key: None,
//             rotation_keypair: None,
//             rotation_secret_key: "".to_string(),
//             invite_code: "".to_string(),
//             privacy_policy_lock: Arc::new(Default::default()),
//             terms_of_service_lock: Arc::new(Default::default()),
//             invite_code_required: Arc::new(Default::default()),
//             page,
//             current_step: MigrationStep::StepOne,
//             repo_file_dialog: Default::default(),
//             picked_repo_file: None,
//             service_jwt_token: None,
//             temporary_signing_keypair,
//             did: "".to_string(),
//         }
//     }
//
//     fn update_pds(&mut self) {
//         let error = self.error.clone();
//         let terms_of_service_lock = self.terms_of_service_lock.clone();
//         let privacy_policy_lock = self.privacy_policy_lock.clone();
//         let invite_code_required = self.invite_code_required.clone();
//         let new_pds_host = self.new_pds_host.clone();
//         tokio::spawn(async move {
//             match fetch_tos_and_privacy_policy(new_pds_host).await {
//                 Ok(result) => {
//                     let mut privacy_policy_write = privacy_policy_lock.write().await;
//                     *privacy_policy_write = result.privacy_policy;
//                     let mut terms_of_service_lock = terms_of_service_lock.write().await;
//                     *terms_of_service_lock = result.terms_of_service;
//                     let mut invite_code_required_write = invite_code_required.write().await;
//                     *invite_code_required_write = result.invite_code_required;
//                 }
//                 Err(e) => {
//                     let mut errors = error.write().await;
//                     errors.push(e);
//                 }
//             }
//         });
//     }
//
//     fn show_step_one(&mut self, ui: &mut Ui, ctx: &egui::Context) {
//         styles::render_subtitle(ui, ctx, "Migration Step 1!");
//         ui.vertical_centered(|ui| {
//             ui.horizontal(|ui| {
//                 if self.pds_selected {
//                     styles::render_input_disabled(
//                         ui,
//                         "New PDS Host",
//                         &mut self.new_pds_host,
//                         false,
//                         Some("https://northsky.social"),
//                     );
//                     styles::render_button(ui, ctx, "Edit", || self.pds_selected = false);
//                 } else {
//                     styles::render_input(
//                         ui,
//                         "New PDS Host",
//                         &mut self.new_pds_host,
//                         false,
//                         Some("https://northsky.social"),
//                     );
//                     styles::render_button(ui, ctx, "Update", || {
//                         self.pds_selected = true;
//                         self.update_pds();
//                     });
//                 }
//             });
//             if self.pds_selected {
//                 styles::render_input(ui, "Email", &mut self.new_email, false, None);
//                 styles::render_input(
//                     ui,
//                     "Handle",
//                     &mut self.new_handle,
//                     false,
//                     Some("user.northsky.social"),
//                 );
//                 styles::render_input(ui, "Password", &mut self.new_password, true, None);
//                 styles::render_input(
//                     ui,
//                     "Invite Code (Leave Blank if None)",
//                     &mut self.invite_code,
//                     false,
//                     None,
//                 );
//
//                 let privacy_policy = {
//                     let privacy_policy = self.privacy_policy_lock.blocking_read();
//                     let value = privacy_policy.clone();
//                     value.unwrap_or("".to_string())
//                 };
//                 let terms_of_service = {
//                     let terms_of_service = self.privacy_policy_lock.blocking_read().clone();
//                     let value = terms_of_service.clone();
//                     value.unwrap_or("".to_string())
//                 };
//                 if !privacy_policy.is_empty() || !terms_of_service.is_empty() {
//                     ui.horizontal_wrapped(|ui| {
//                         ui.spacing_mut().item_spacing.x = 0.0;
//                         ui.label("By creating an account you agree to the ");
//
//                         if !terms_of_service.is_empty() {
//                             ui.hyperlink_to("Terms of Service", terms_of_service);
//                             if !privacy_policy.is_empty() {
//                                 ui.label(" and ");
//                                 ui.hyperlink_to("Privacy Policy", privacy_policy);
//                                 ui.label(".");
//                             } else {
//                                 ui.label(".");
//                             }
//                         } else {
//                             ui.hyperlink_to("Privacy Policy", privacy_policy);
//                             ui.label(".");
//                         }
//                     });
//                 }
//                 styles::render_button(ui, ctx, "Next", || {
//                     self.current_step = MigrationStep::StepTwo;
//                 });
//             }
//         });
//     }
//
//     fn show_step_two(&mut self, ui: &mut Ui, ctx: &egui::Context) {
//         styles::render_subtitle(ui, ctx, "Migration Step 2!");
//         ui.vertical_centered(|ui| {
//             styles::render_input(ui, "Enter your DID", &mut self.did, true, None);
//             styles::render_button(ui, ctx, "Select your repo", || {
//                 self.repo_file_dialog.pick_file();
//             });
//             ui.label(format!("Picked file: {:?}", self.picked_repo_file));
//
//             if let Some(path) = self.repo_file_dialog.take_picked() {
//                 self.picked_repo_file = Some(path.to_path_buf());
//             }
//
//             styles::render_input(
//                 ui,
//                 "Rotation Key (private)",
//                 &mut self.rotation_secret_key,
//                 true,
//                 None,
//             );
//             styles::render_button(ui, ctx, "Next", || {
//                 self.submit();
//             });
//         });
//     }
//
//     fn submit(&mut self) {
//         let did = self.did.clone();
//         let rotation_keypair = self.rotation_keypair.unwrap();
//         let public_rotation_key = rotation_keypair.public_key();
//         let formatted_public_rotation_key = encode_did_key(&public_rotation_key);
//         let secret_rotation_key = rotation_keypair.secret_key();
//         let pds_session = self.pds_session.clone().unwrap();
//         let new_pds_host = self.new_pds_host.clone();
//         let temporary_signing_keypair = self.temporary_signing_keypair;
//         let signing_public_key = temporary_signing_keypair.public_key();
//         let formatted_public_signing_key = encode_did_key(&signing_public_key);
//         let _signing_secret_key = temporary_signing_keypair.secret_key();
//         let new_handle = self.new_handle.clone();
//         let new_password = self.new_password.clone();
//         let new_email = self.new_email.clone();
//         let invite_code = if self.invite_code.is_empty() {
//             None
//         } else {
//             Some(self.invite_code.clone())
//         };
//         let _error_lock = self.error.clone();
//
//         tokio::spawn(async move {
//             let audit_log = get_plc_audit_log(did.as_str()).await;
//             let entry = audit_log.last().unwrap();
//             let last_op = entry.operation.clone();
//
//             // Update PLC with new signing key and new PDS
//             let operation = create_update_op(
//                 last_op,
//                 &secret_rotation_key,
//                 |normalized: PlcOperation| -> PlcOperation {
//                     let mut updated = normalized.clone();
//                     updated.also_known_as = vec![format!("at://{}", new_handle)];
//                     updated.rotation_keys.clear();
//                     updated
//                         .rotation_keys
//                         .push(formatted_public_rotation_key.clone());
//                     updated.services.remove("atproto_pds");
//                     updated.services.insert(
//                         "atproto_pds".to_string(),
//                         PlcOpService {
//                             r#type: "AtprotoPersonalDataServer".to_string(),
//                             endpoint: new_pds_host.to_string(),
//                         },
//                     );
//                     updated.verification_methods.remove("atproto");
//                     updated
//                         .verification_methods
//                         .insert("atproto".to_string(), formatted_public_signing_key.clone());
//                     updated
//                 },
//             )
//             .await;
//             submit_plc_update(did.as_str(), operation.clone()).await;
//
//             //Generate Service Auth Token with new signing key
//             let jwt_access_token = create_service_auth(
//                 did.as_str(),
//                 new_pds_host.as_str(),
//                 temporary_signing_keypair.secret_key(),
//             )
//             .await;
//
//             // Create Account on New PDS
//             let request = CreateAccountRequest {
//                 did: Did::from_str(did.as_str()).unwrap(),
//                 email: Some(new_email.clone()),
//                 handle: new_handle.clone(),
//                 invite_code: invite_code.clone(),
//                 password: Some(new_password.clone()),
//                 recovery_key: None,
//                 verification_code: None,
//                 verification_phone: None,
//                 plc_op: None,
//                 token: jwt_access_token,
//             };
//             create_account(new_pds_host.as_str(), &request)
//                 .await
//                 .unwrap();
//
//             let agent = BskyAgent::builder().build().await.unwrap();
//             agent.configure_endpoint(new_pds_host.clone());
//             agent
//                 .login(new_handle.as_str(), new_password.as_str())
//                 .await
//                 .unwrap();
//
//             // Standard migration stuff
//             tracing::info!("Importing repo to new PDS");
//             match import_repo(pds_session.clone()).await {
//                 Ok(_) => {
//                     tracing::info!("Repo imported successfully");
//                 }
//                 Err(e) => {
//                     panic!("Error importing repo: {e}");
//                 }
//             }
//             tracing::info!("Importing blobs to new PDS");
//             match upload_blobs(pds_session.clone()).await {
//                 Ok(_) => {
//                     tracing::info!("Importing blobs successful");
//                 }
//                 Err(e) => {
//                     tracing::error!("Error uploading blobs: {}", e);
//                 }
//             }
//             tracing::info!("Migrating preferences from old to new PDS");
//             match migrate_preferences(pds_session.clone()).await {
//                 Ok(_) => {
//                     tracing::info!("Preferences migrated successfully");
//                 }
//                 Err(_e) => {}
//             }
//
//             // Update PLC
//             let access_token = agent.get_session().await.unwrap().access_jwt.clone();
//             let recommended = get_recommended(new_pds_host.as_str(), access_token.as_str()).await;
//             let recommended_rotation_keys = recommended.rotation_keys.clone();
//             let recommended_verification_methods = recommended.verification_methods.clone();
//             let new_verification_key = recommended_verification_methods
//                 .get("atproto")
//                 .unwrap()
//                 .clone();
//             let audit_log = get_plc_audit_log(did.as_str()).await;
//             let entry = audit_log.last().unwrap();
//             let last_op = entry.operation.clone();
//             let rotation_keys = {
//                 let mut rotation_keys = last_op.rotation_keys.clone();
//                 for key in recommended_rotation_keys {
//                     rotation_keys.push(key);
//                 }
//                 rotation_keys
//             };
//             let operation = create_update_op(
//                 last_op,
//                 &secret_rotation_key,
//                 |normalized: PlcOperation| -> PlcOperation {
//                     let mut rotation_keys = rotation_keys.clone();
//                     let mut updated = normalized.clone();
//                     updated.rotation_keys.append(&mut rotation_keys);
//                     updated.verification_methods.remove("atproto");
//                     updated
//                         .verification_methods
//                         .insert("atproto".to_string(), new_verification_key.clone());
//                     updated
//                 },
//             )
//             .await;
//             submit_plc_update(did.as_str(), operation.clone()).await;
//
//             // Activate accounts
//             tracing::info!("Activating new account");
//             agent
//                 .api
//                 .com
//                 .atproto
//                 .server
//                 .activate_account()
//                 .await
//                 .unwrap();
//         });
//     }
// }
//
// impl Screen for MigrateWithoutPds {
//     fn ui(&mut self, ui: &mut Ui, ctx: &egui::Context) {
//         match self.current_step {
//             MigrationStep::StepOne => {
//                 self.show_step_one(ui, ctx);
//             }
//             MigrationStep::StepTwo => {
//                 self.show_step_two(ui, ctx);
//             }
//         }
//     }
//
//     fn name(&self) -> ScreenType {
//         ScreenType::MigrateWithoutPds
//     }
// }
//
// async fn submit_plc_update(did: &str, op: PlcOperation) {
//     send_plc_operation(did, op).await;
// }
//
// async fn create_service_auth(did: &str, new_pds_host: &str, secret_key: SecretKey) -> String {
//     let aud = new_pds_host.replace("https://", "did:web:");
//     let lxm = Some("com.atproto.server.createAccount".to_string());
//     let service_jwt = create_service_jwt(ServiceJwtParams {
//         iss: did.to_string(),
//         aud,
//         exp: None,
//         lxm,
//         jti: None,
//         secret_key,
//     })
//     .await;
//     service_jwt
// }
//
// #[cfg(test)]
// mod tests {
//     use super::*;
//     use crate::get_keys_from_private_key_str;
//     use bsky_sdk::api::agent::Configure;
//     use pdsmigration_common::agent::{get_plc_audit_log, get_recommended};
//
//     #[tokio::test]
//     async fn test_sign_operation() {
//         let did = "did:plc:isbzzzuos6xe4ira4tkgbrcm".to_string();
//
//         let secp = Secp256k1::new();
//         let temporary_signing_keypair = Keypair::new(&secp, &mut rand::rng());
//
//         let (secret_rotation_key, public_rotation_key) = get_keys_from_private_key_str(
//             "150bb11077f1f4c7f7008c1cafae180e9da61c0010640933ba2cf906a8568c0d".to_string(),
//         );
//         let formatted_public_rotation_key = encode_did_key(&public_rotation_key);
//         let new_pds_host = "https://northsky.social".to_string();
//         let signing_public_key = temporary_signing_keypair.public_key();
//         let formatted_public_signing_key = encode_did_key(&signing_public_key);
//
//         let audit_log = get_plc_audit_log(did.as_str()).await;
//         let entry = audit_log.last().unwrap();
//         let last_op = entry.operation.clone();
//
//         // Update PLC with new signing key and new PDS
//         let operation = create_update_op(
//             last_op,
//             &secret_rotation_key,
//             |normalized: PlcOperation| -> PlcOperation {
//                 let mut updated = normalized.clone();
//                 updated.also_known_as = vec!["at://testripp104.northsky.social".to_string()];
//                 updated.rotation_keys.clear();
//                 updated
//                     .rotation_keys
//                     .push(formatted_public_rotation_key.clone());
//                 updated.services.remove("atproto_pds");
//                 updated.services.insert(
//                     "atproto_pds".to_string(),
//                     PlcOpService {
//                         r#type: "AtprotoPersonalDataServer".to_string(),
//                         endpoint: new_pds_host.to_string(),
//                     },
//                 );
//                 updated.verification_methods.remove("atproto");
//                 updated
//                     .verification_methods
//                     .insert("atproto".to_string(), formatted_public_signing_key.clone());
//                 updated
//             },
//         )
//         .await;
//         submit_plc_update(did.as_str(), operation.clone()).await;
//
//         // //Generate Service Auth Token with new signing key
//         // let jwt_access_token = create_service_auth(
//         //     did.as_str(),
//         //     new_pds_host.as_str(),
//         //     temporary_signing_keypair.secret_key(),
//         // )
//         // .await;
//
//         // // Create Account on New PDS
//         // let request = CreateAccountRequest {
//         //     did: Did::from_str(did.as_str()).unwrap(),
//         //     email: Some("testripp104@northsky.social".to_string()),
//         //     handle: "testripp104.northsky.social".to_string(),
//         //     invite_code: Some("northsky-social-gf5s3-kqvjg".to_string()),
//         //     password: Some("password123".to_string()),
//         //     recovery_key: None,
//         //     verification_code: None,
//         //     verification_phone: None,
//         //     plc_op: None,
//         //     token: jwt_access_token,
//         // };
//         // create_account("https://northsky.social", &request).await.unwrap();
//
//         let agent = BskyAgent::builder().build().await.unwrap();
//         agent.configure_endpoint("https://northsky.social".to_string());
//         agent
//             .login("testripp104.northsky.social", "password123")
//             .await
//             .unwrap();
//
//         // // Standard migration stuff
//         // tracing::info!("Importing repo to new PDS");
//         // match import_repo(pds_session.clone()).await {
//         //     Ok(_) => {
//         //         tracing::info!("Repo imported successfully");
//         //     }
//         //     Err(e) => {
//         //         panic!("Error importing repo: {e}");
//         //     }
//         // }
//         // tracing::info!("Importing blobs to new PDS");
//         // match upload_blobs(pds_session.clone()).await {
//         //     Ok(_) => {
//         //         tracing::info!("Importing blobs successful");
//         //     }
//         //     Err(e) => {
//         //         tracing::error!("Error uploading blobs: {}", e);
//         //     }
//         // }
//         // tracing::info!("Migrating preferences from old to new PDS");
//         // match migrate_preferences(pds_session.clone()).await {
//         //     Ok(_) => {
//         //         tracing::info!("Preferences migrated successfully");
//         //     }
//         //     Err(e) => {}
//         // }
//         //
//         // // Update PLC
//         let access_token = agent.get_session().await.unwrap().access_jwt.clone();
//         let recommended = get_recommended(new_pds_host.as_str(), access_token.as_str()).await;
//         let recommended_rotation_keys = recommended.rotation_keys.clone();
//         let recommended_verification_methods = recommended.verification_methods.clone();
//         let new_verification_key = recommended_verification_methods
//             .get("atproto")
//             .unwrap()
//             .clone();
//         let audit_log = get_plc_audit_log(did.as_str()).await;
//         let entry = audit_log.last().unwrap();
//         let last_op = entry.operation.clone();
//         let rotation_keys = {
//             let mut rotation_keys = last_op.rotation_keys.clone();
//             for key in recommended_rotation_keys {
//                 rotation_keys.push(key);
//             }
//             rotation_keys
//         };
//         let operation = create_update_op(
//             last_op,
//             &secret_rotation_key,
//             |normalized: PlcOperation| -> PlcOperation {
//                 let mut rotation_keys = rotation_keys.clone();
//                 let mut updated = normalized.clone();
//                 updated.rotation_keys.append(&mut rotation_keys);
//                 updated.verification_methods.remove("atproto");
//                 updated
//                     .verification_methods
//                     .insert("atproto".to_string(), new_verification_key.clone());
//                 updated
//             },
//         )
//         .await;
//         submit_plc_update(did.as_str(), operation.clone()).await;
//
//         // Activate accounts
//         tracing::info!("Activating new account");
//         agent
//             .api
//             .com
//             .atproto
//             .server
//             .activate_account()
//             .await
//             .unwrap();
//     }
// }
