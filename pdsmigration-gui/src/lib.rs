use crate::agent::login_helper2;
use crate::app::PdsMigrationApp;
use crate::errors::GuiError;
use crate::session::session_config::{PdsSession, SessionConfig};
use bsky_sdk::api::agent::Configure;
use bsky_sdk::BskyAgent;
use multibase::Base::Base58Btc;
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
use tracing::Level;
use tracing_subscriber::filter;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use zip::write::SimpleFileOptions;
use zip::{AesMode, ZipWriter};

pub mod agent;
pub mod app;
pub mod error_window;
pub mod errors;
pub mod screens;
pub mod session;
pub mod styles;
pub mod success_window;

#[derive(PartialEq, Clone)]
pub enum ScreenType {
    Basic,
    OldLogin,
    NewLogin,
    AccountCreate,
    MigratePLC,
    Success,
    ExportBlobs,
    ImportBlobs,
    MigratePreferences,
    ActiveAccounts,
    DoesAccountExist,
    CreateNewAccount,
    ExportRepo,
    ImportRepo,
}

#[tracing::instrument(skip(session_config))]
pub async fn activate_account(session_config: SessionConfig) -> Result<(), GuiError> {
    let pds_host = session_config.host().to_string();
    let token = session_config.access_token().to_string();
    let did = session_config.did().to_string();

    tracing::info!("Activating Account started");
    let request = ActivateAccountRequest {
        pds_host,
        did,
        token,
    };
    match pdsmigration_common::activate_account_api(request).await {
        Ok(_) => {
            tracing::info!("Activating Account completed");
            Ok(())
        }
        Err(pds_error) => {
            tracing::error!("Error activating account: {pds_error}");
            Err(GuiError::Runtime)
        }
    }
}

#[tracing::instrument(skip(session_config))]
pub async fn deactivate_account(session_config: SessionConfig) -> Result<(), GuiError> {
    let pds_host = session_config.host().to_string();
    let token = session_config.access_token().to_string();
    let did = session_config.did().to_string();

    tracing::info!("Deactivating Account started");
    let request = DeactivateAccountRequest {
        pds_host,
        did,
        token,
    };
    match pdsmigration_common::deactivate_account_api(request).await {
        Ok(_) => {
            tracing::info!("Deactivating Account completed");
            Ok(())
        }
        Err(pds_error) => {
            tracing::error!("Error deactivating account: {pds_error}");
            Err(GuiError::Runtime)
        }
    }
}

#[tracing::instrument]
pub fn generate_recovery_key(user_recovery_key_password: String) -> Result<String, GuiError> {
    let secp = Secp256k1::new();
    let (secret_key, public_key) = secp.generate_keypair(&mut rand::rng());
    let pk_compact = public_key.serialize();
    let pk_wrapped = multicodec_wrap(pk_compact.to_vec());
    let pk_multibase = multibase::encode(Base58Btc, pk_wrapped.as_slice());
    let public_key_str = format!("did:key:{pk_multibase}");

    let sk_compact = secret_key.secret_bytes().to_vec();
    let sk_wrapped = multicodec_wrap(sk_compact.to_vec());
    let sk_multibase = multibase::encode(Base58Btc, sk_wrapped.as_slice());
    let secret_key_str = format!("did:key:{sk_multibase}");

    let path = std::path::Path::new("RotationKey.zip");
    let file = match std::fs::File::create(path) {
        Ok(file) => file,
        Err(e) => {
            tracing::error!("Error creating file: {e}");
            return Err(GuiError::Runtime);
        }
    };

    let mut zip = ZipWriter::new(file);

    let options = SimpleFileOptions::default()
        .compression_method(zip::CompressionMethod::Stored)
        .with_aes_encryption(AesMode::Aes256, user_recovery_key_password.as_str());
    match zip.start_file("RotationKey", options) {
        Ok(_) => {}
        Err(e) => {
            tracing::error!("Error starting file: {e}");
            return Err(GuiError::Runtime);
        }
    }
    match zip.write_all(secret_key_str.as_bytes()) {
        Ok(_) => {}
        Err(e) => {
            tracing::error!("Error writing file: {e}");
            return Err(GuiError::Runtime);
        }
    }

    match zip.finish() {
        Ok(_) => {}
        Err(e) => {
            tracing::error!("Error finishing file: {e}");
            return Err(GuiError::Runtime);
        }
    }
    Ok(public_key_str)
}

#[tracing::instrument(skip(session_config))]
pub async fn request_token(session_config: SessionConfig) -> Result<(), GuiError> {
    let pds_host = session_config.host().to_string();
    let token = session_config.access_token().to_string();
    let did = session_config.did().to_string();

    tracing::info!("Requesting Token started");
    let request = RequestTokenRequest {
        pds_host,
        did,
        token,
    };
    match pdsmigration_common::request_token_api(request).await {
        Ok(_) => {
            tracing::info!("Requesting Token completed");
            Ok(())
        }
        Err(pds_error) => {
            tracing::error!("Error requesting token: {pds_error}");
            Err(GuiError::Runtime)
        }
    }
}

#[tracing::instrument(skip(pds_session))]
pub async fn migrate_preferences(pds_session: PdsSession) -> Result<(), GuiError> {
    let did = match pds_session.did().clone() {
        None => {
            return Err(GuiError::Other);
        }
        Some(did) => did.to_string(),
    };
    let old_session_config = match &pds_session.old_session_config() {
        None => {
            return Err(GuiError::Other);
        }
        Some(config) => config,
    };
    let new_session_config = match &pds_session.new_session_config() {
        None => {
            return Err(GuiError::Other);
        }
        Some(config) => config,
    };
    let origin = old_session_config.host().to_string();
    let destination = new_session_config.host().to_string();
    let origin_token = old_session_config.access_token().to_string();
    let destination_token = new_session_config.access_token().to_string();

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
            Ok(())
        }
        Err(pds_error) => {
            tracing::error!("Error migrating Preferences: {pds_error}");
            Err(GuiError::Runtime)
        }
    }
}

pub fn multicodec_wrap(bytes: Vec<u8>) -> Vec<u8> {
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

#[tracing::instrument(skip(pds_session))]
pub async fn migrate_plc_via_pds(
    pds_session: PdsSession,
    plc_signing_token: String,
    user_recovery_key: Option<String>,
) -> Result<(), GuiError> {
    let did = match pds_session.did().clone() {
        None => {
            return Err(GuiError::Other);
        }
        Some(did) => did.to_string(),
    };
    let old_session_config = match &pds_session.old_session_config() {
        None => {
            return Err(GuiError::Other);
        }
        Some(config) => config,
    };
    let new_session_config = match &pds_session.new_session_config() {
        None => {
            return Err(GuiError::Other);
        }
        Some(config) => config,
    };
    let origin = old_session_config.host().to_string();
    let destination = new_session_config.host().to_string();
    let origin_token = old_session_config.access_token().to_string();
    let destination_token = new_session_config.access_token().to_string();

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
            return Ok(());
        }
        Err(_pds_error) => {
            tracing::error!("Error migrating PLC: {_pds_error}");
            return Err(GuiError::Runtime);
        }
    }
}

#[tracing::instrument(skip(pds_session))]
pub async fn upload_blobs(pds_session: PdsSession) -> Result<(), GuiError> {
    let did = match pds_session.did().clone() {
        None => {
            return Err(GuiError::Other);
        }
        Some(did) => did.to_string(),
    };
    let new_session_config = match &pds_session.new_session_config() {
        None => {
            return Err(GuiError::Other);
        }
        Some(config) => config,
    };
    let pds_host = new_session_config.host().to_string();
    let token = new_session_config.access_token().to_string();

    tracing::info!("Uploading Blobs started");
    let request = UploadBlobsRequest {
        pds_host,
        did,
        token,
    };
    match pdsmigration_common::upload_blobs_api(request).await {
        Ok(_) => {
            tracing::info!("Uploading Blobs completed");
            return Ok(());
        }
        Err(_pds_error) => {
            tracing::error!("Error uploading blobs: {_pds_error}");
            return Err(GuiError::Runtime);
        }
    }
}

#[tracing::instrument(skip(pds_session))]
pub async fn export_all_blobs(pds_session: PdsSession) -> Result<(), GuiError> {
    let did = match pds_session.did().clone() {
        None => {
            return Err(GuiError::Other);
        }
        Some(did) => did.to_string(),
    };
    let old_session_config = match &pds_session.old_session_config() {
        None => {
            return Err(GuiError::Other);
        }
        Some(config) => config,
    };
    let old_pds_host = old_session_config.host().to_string();
    let old_token = old_session_config.access_token().to_string();

    tracing::info!("Exporting All Blobs started");
    let request = ExportAllBlobsRequest {
        origin: old_pds_host,
        did,
        origin_token: old_token,
    };
    match pdsmigration_common::export_all_blobs_api(request).await {
        Ok(_) => {
            tracing::info!("Exporting All Blobs completed");
            return Ok(());
        }
        Err(pds_error) => match pds_error {
            PdsError::Validation => {
                tracing::error!(
                    "Error exporting all blobs, validation error: {:?}",
                    pds_error
                );
                return Err(GuiError::Other);
            }
            _ => {
                tracing::error!("Error exporting all blobs: {:?}", pds_error);
                return Err(GuiError::Runtime);
            }
        },
    }
}

#[tracing::instrument(skip(pds_session))]
pub async fn export_missing_blobs(pds_session: PdsSession) -> Result<(), GuiError> {
    let did = match pds_session.did().clone() {
        None => {
            tracing::error!("No DID found");
            return Err(GuiError::Other);
        }
        Some(did) => did.to_string(),
    };
    let old_session_config = match &pds_session.old_session_config() {
        None => {
            return Err(GuiError::Other);
        }
        Some(config) => config,
    };
    let new_session_config = match &pds_session.new_session_config() {
        None => {
            return Err(GuiError::Other);
        }
        Some(config) => config,
    };
    let old_pds_host = old_session_config.host().to_string();
    let new_pds_host = new_session_config.host().to_string();
    let old_token = old_session_config.access_token().to_string();
    let new_token = new_session_config.access_token().to_string();

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
            return Ok(());
        }
        Err(pds_error) => match pds_error {
            PdsError::Validation => {
                tracing::error!(
                    "Error exporting missing blobs, validation error: {:?}",
                    pds_error
                );
                return Err(GuiError::Other);
            }
            _ => {
                tracing::error!("Error exporting missing blobs: {:?}", pds_error);
                return Err(GuiError::Runtime);
            }
        },
    }
}

#[tracing::instrument(skip(pds_session))]
pub async fn import_repo(pds_session: PdsSession) -> Result<(), GuiError> {
    let did = match pds_session.did().clone() {
        None => {
            tracing::error!("No DID found");
            return Err(GuiError::Other);
        }
        Some(did) => did.to_string(),
    };
    let new_session_config = match &pds_session.new_session_config() {
        None => {
            tracing::error!("No new session config found");
            return Err(GuiError::Other);
        }
        Some(config) => config,
    };
    let pds_host = new_session_config.host().to_string();
    let token = new_session_config.access_token().to_string();

    tracing::info!("Importing Repo started");
    let request = ImportPDSRequest {
        pds_host,
        did,
        token,
    };
    match pdsmigration_common::import_pds_api(request).await {
        Ok(_) => {
            tracing::info!("Importing Repo completed");
            return Ok(());
        }
        Err(pds_error) => {
            tracing::error!("Error importing repo: {:?}", pds_error);
            return Err(GuiError::Runtime);
        }
    }
}

#[tracing::instrument(skip(pds_session))]
pub async fn export_repo(pds_session: PdsSession) -> Result<(), GuiError> {
    let did = match pds_session.did().clone() {
        None => {
            tracing::error!("No DID found");
            return Err(GuiError::Other);
        }
        Some(did) => did.to_string(),
    };

    let old_session_config = match &pds_session.old_session_config() {
        None => {
            tracing::error!("No old session config found");
            return Err(GuiError::Other);
        }
        Some(config) => config,
    };
    let pds_host = old_session_config.host().to_string();
    let token = old_session_config.access_token().to_string();

    tracing::info!("Exporting Repo started");
    let request = ExportPDSRequest {
        pds_host,
        did,
        token,
    };
    match pdsmigration_common::export_pds_api(request).await {
        Ok(_) => {
            tracing::info!("Exporting Repo completed");
            Ok(())
        }
        Err(pds_error) => {
            tracing::error!("Error exporting repo: {:?}", pds_error);
            match pds_error {
                PdsError::Login => Err(GuiError::InvalidLogin),
                PdsError::Runtime => Err(GuiError::Runtime),
                PdsError::AccountExport => Err(GuiError::Other),
                _ => Err(GuiError::Other),
            }
        }
    }
}

pub struct DescribePDS {
    pub terms_of_service: Option<String>,
    pub privacy_policy: Option<String>,
    pub invite_code_required: bool,
}

#[tracing::instrument]
pub async fn fetch_tos_and_privacy_policy(new_pds_host: String) -> Result<DescribePDS, GuiError> {
    tracing::info!(
        "Fetching TOS and Privacy Policy from new PDS: {}",
        new_pds_host
    );
    let bsky_agent = BskyAgent::builder().build().await.unwrap();
    bsky_agent.configure_endpoint(new_pds_host);
    match bsky_agent.api.com.atproto.server.describe_server().await {
        Ok(result) => match result.links.clone() {
            None => Ok(DescribePDS {
                terms_of_service: None,
                privacy_policy: None,
                invite_code_required: result.invite_code_required.unwrap_or(false),
            }),
            Some(links) => Ok(DescribePDS {
                terms_of_service: links.terms_of_service.clone(),
                privacy_policy: links.privacy_policy.clone(),
                invite_code_required: result.invite_code_required.unwrap_or(false),
            }),
        },
        Err(error) => {
            tracing::error!(
                "Error fetching TOS and Privacy Policy from new PDS: {:?}",
                error
            );
            Err(GuiError::Runtime)
        }
    }
}

pub struct CreateAccountParameters {
    pds_session: PdsSession,
    new_email: String,
    new_pds_host: String,
    new_password: String,
    new_handle: String,
    invite_code: String,
}

#[tracing::instrument(skip(parameters))]
pub async fn create_account(parameters: CreateAccountParameters) -> Result<PdsSession, GuiError> {
    let mut pds_session = parameters.pds_session.clone();
    let old_session_config = match &pds_session.old_session_config() {
        None => return Err(GuiError::Other),
        Some(session_config) => session_config.clone(),
    };
    let did = match pds_session.did().clone() {
        None => return Err(GuiError::Other),
        Some(did) => did.to_string(),
    };
    let email = parameters.new_email.clone();
    let new_pds_host = parameters.new_pds_host.clone();
    let aud = new_pds_host.replace("https://", "did:web:");

    let password = parameters.new_password.clone();
    let invite_code = parameters.invite_code.clone();
    let handle = parameters.new_handle.clone();
    tracing::info!("Creating Account started");
    let service_auth_request = ServiceAuthRequest {
        pds_host: old_session_config.host().to_string(),
        aud,
        did: did.clone(),
        token: old_session_config.access_token().to_string(),
    };
    let service_token = match pdsmigration_common::get_service_auth_api(service_auth_request).await
    {
        Ok(res) => res,
        Err(_pds_error) => return Err(GuiError::Runtime),
    };

    let create_account_request = CreateAccountApiRequest {
        email,
        handle: handle.clone(),
        invite_code,
        password: password.clone(),
        token: service_token,
        pds_host: new_pds_host.clone(),
        did,
        recovery_key: None,
    };
    match pdsmigration_common::create_account_api(create_account_request).await {
        Ok(_) => {
            tracing::info!("Creating Account completed");
            let bsky_agent = BskyAgent::builder().build().await.unwrap();
            match login_helper2(
                &bsky_agent,
                new_pds_host.as_str(),
                handle.as_str(),
                password.as_str(),
            )
            .await
            {
                Ok(res) => {
                    tracing::info!("Login successful");
                    let access_token = res.access_jwt.clone();
                    let refresh_token = res.refresh_jwt.clone();
                    let did = res.did.as_str().to_string();
                    pds_session.create_new_session(
                        did.as_str(),
                        access_token.as_str(),
                        refresh_token.as_str(),
                        new_pds_host.as_str(),
                    );
                    Ok(pds_session)
                }
                Err(e) => {
                    tracing::error!("Error logging in: {e}");
                    Err(GuiError::Other)
                }
            }
        }
        Err(pds_error) => {
            tracing::error!("Error creating account: {pds_error}");
            Err(GuiError::Runtime)
        }
    }
}

pub fn run() -> eframe::Result {
    use std::time::Duration;
    use tokio::runtime::Runtime;

    let filter = filter::Targets::new().with_target("pdsmigration", Level::INFO);

    let collector = egui_tracing::EventCollector::default();
    tracing_subscriber::registry()
        .with(collector.clone())
        .with(filter)
        .init();

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

    let icon_data =
        eframe::icon_data::from_png_bytes(include_bytes!("../assets/Northsky-Icon_Color.png"))
            .expect("The icon data must be valid");

    let options = eframe::NativeOptions {
        viewport: {
            egui::ViewportBuilder {
                icon: Some(Arc::new(icon_data)),
                ..Default::default()
            }
        },
        ..Default::default()
    };

    eframe::run_native(
        "PDS Migration Tool",
        options,
        Box::new(|cc| {
            egui_extras::install_image_loaders(&cc.egui_ctx);
            styles::setup_fonts(&cc.egui_ctx);
            Ok(Box::new(PdsMigrationApp::new(cc, collector)))
        }),
    )
}
