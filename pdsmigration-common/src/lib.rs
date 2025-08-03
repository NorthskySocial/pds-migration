use crate::agent::{
    account_export, account_import, activate_account, create_account, deactivate_account,
    export_preferences, get_blob, get_plc_audit_log, get_recommended, get_service_auth,
    import_preferences, list_all_blobs, login_helper, missing_blobs, recommended_plc,
    request_token, sign_plc, submit_plc, upload_blob, PlcOpService, PlcOperation,
};
use crate::errors::PdsError;
use bsky_sdk::api::agent::Configure;
use bsky_sdk::api::types::string::Did;
use bsky_sdk::BskyAgent;
use ipld_core::cid::Cid;
use multibase::Base::Base58Btc;
use secp256k1::{PublicKey, Secp256k1};
use serde::{Deserialize, Serialize};
use std::io::ErrorKind;

pub mod agent;
pub mod errors;

#[derive(Debug, Deserialize, Serialize)]
pub struct ServiceAuthRequest {
    pub pds_host: String,
    pub aud: String,
    pub did: String,
    pub token: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ServiceAuthResponse {
    pub token: String,
}

pub async fn get_service_auth_api(req: ServiceAuthRequest) -> Result<String, PdsError> {
    let agent = BskyAgent::builder().build().await.unwrap();
    login_helper(
        &agent,
        req.pds_host.as_str(),
        req.did.as_str(),
        req.token.as_str(),
    )
    .await?;
    let token = get_service_auth(&agent, req.aud.as_str()).await?;
    Ok(token)
}

#[derive(Debug, Deserialize, Serialize)]
pub struct CreateAccountApiRequest {
    pub email: String,
    pub handle: String,
    pub invite_code: String,
    pub password: String,
    pub token: String,
    pub pds_host: String,
    pub did: String,
    #[serde(skip_serializing_if = "core::option::Option::is_none")]
    pub recovery_key: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct CreateAccountRequest {
    pub did: Did,
    pub email: Option<String>,
    pub handle: String,
    pub invite_code: Option<String>,
    pub password: Option<String>,
    pub recovery_key: Option<String>,
    pub verification_code: Option<String>,
    pub verification_phone: Option<String>,
    pub plc_op: Option<String>,
    pub token: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct CreateAccountWithoutPDSRequest {
    pub did: Did,
    pub email: Option<String>,
    pub handle: String,
    pub invite_code: Option<String>,
    pub password: Option<String>,
    pub recovery_key: Option<String>,
    pub verification_code: Option<String>,
    pub verification_phone: Option<String>,
    pub plc_op: Option<String>,
}

#[tracing::instrument(skip(req))]
pub async fn create_account_api(req: CreateAccountApiRequest) -> Result<(), PdsError> {
    create_account(
        req.pds_host.as_str(),
        &CreateAccountRequest {
            did: req.did.parse().unwrap(),
            email: Some(req.email.clone()),
            handle: req.handle.parse().unwrap(),
            invite_code: Some(req.invite_code.clone()),
            password: Some(req.password.clone()),
            recovery_key: req.recovery_key.clone(),
            verification_code: Some(String::from("")),
            verification_phone: None,
            plc_op: None,
            token: req.token.clone(),
        },
    )
    .await?;
    Ok(())
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ExportPDSRequest {
    pub pds_host: String,
    pub did: String,
    pub token: String,
}

#[tracing::instrument]
pub async fn export_pds_api(req: ExportPDSRequest) -> Result<(), PdsError> {
    let agent = BskyAgent::builder().build().await.map_err(|error| {
        tracing::error!("{}", error.to_string());
        PdsError::Runtime
    })?;
    let session = login_helper(
        &agent,
        req.pds_host.as_str(),
        req.did.as_str(),
        req.token.as_str(),
    )
    .await?;
    account_export(&agent, &session.did).await?;
    Ok(())
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ImportPDSRequest {
    pub pds_host: String,
    pub did: String,
    pub token: String,
}

#[tracing::instrument]
pub async fn import_pds_api(req: ImportPDSRequest) -> Result<(), PdsError> {
    let agent = BskyAgent::builder().build().await.map_err(|error| {
        tracing::error!("{}", error.to_string());
        PdsError::Runtime
    })?;
    let session = login_helper(
        &agent,
        req.pds_host.as_str(),
        req.did.as_str(),
        req.token.as_str(),
    )
    .await?;
    account_import(
        &agent,
        (session.did.as_str().to_string().replace(":", "-") + ".car").as_str(),
    )
    .await?;
    Ok(())
}

#[derive(Debug, Deserialize, Serialize)]
pub struct MissingBlobsRequest {
    pub pds_host: String,
    pub did: String,
    pub token: String,
}

#[tracing::instrument]
pub async fn missing_blobs_api(req: MissingBlobsRequest) -> Result<String, PdsError> {
    let agent = BskyAgent::builder().build().await.unwrap();
    login_helper(
        &agent,
        req.pds_host.as_str(),
        req.did.as_str(),
        req.token.as_str(),
    )
    .await?;
    let initial_missing_blobs = missing_blobs(&agent).await?;
    let mut missing_blob_cids = Vec::new();
    for blob in &initial_missing_blobs {
        missing_blob_cids.push(Cid::to_string(blob.cid.as_ref()));
    }

    let response = serde_json::to_string(&missing_blob_cids).unwrap();
    Ok(response)
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ExportBlobsRequest {
    pub destination: String,
    pub origin: String,
    pub did: String,
    pub origin_token: String,
    pub destination_token: String,
}

#[tracing::instrument]
pub async fn export_blobs_api(req: ExportBlobsRequest) -> Result<(), PdsError> {
    let agent = BskyAgent::builder().build().await.map_err(|error| {
        tracing::error!("{}", error.to_string());
        PdsError::Runtime
    })?;
    login_helper(
        &agent,
        req.destination.as_str(),
        req.did.as_str(),
        req.destination_token.as_str(),
    )
    .await?;
    let missing_blobs = missing_blobs(&agent).await?;
    let session = login_helper(
        &agent,
        req.origin.as_str(),
        req.did.as_str(),
        req.origin_token.as_str(),
    )
    .await?;
    let mut path = std::env::current_dir().unwrap();
    path.push(session.did.as_str().replace(":", "-"));
    match tokio::fs::create_dir(path.as_path()).await {
        Ok(_) => {}
        Err(e) => {
            if e.kind() != ErrorKind::AlreadyExists {
                tracing::error!("Error creating directory: {:?}", e);
                return Err(PdsError::Validation);
            }
        }
    }
    for missing_blob in &missing_blobs {
        match get_blob(&agent, missing_blob.cid.clone(), session.did.clone()).await {
            Ok(output) => {
                tracing::info!("Successfully fetched missing blob");
                let mut path = std::env::current_dir().unwrap();
                path.push(session.did.as_str().replace(":", "-"));
                path.push(
                    missing_blob
                        .record_uri
                        .as_str()
                        .split("/")
                        .last()
                        .unwrap_or("fallback"),
                );
                tokio::fs::write(path.as_path(), output)
                    .await
                    .map_err(|error| {
                        tracing::error!("{}", error.to_string());
                        PdsError::AccountExport
                    })?;
            }
            Err(_) => {
                tracing::error!("Failed to determine missing blobs");
                // return Err(PdsError::Validation);
            }
        }
    }
    Ok(())
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ExportAllBlobsRequest {
    pub origin: String,
    pub did: String,
    pub origin_token: String,
}

#[tracing::instrument]
pub async fn export_all_blobs_api(req: ExportAllBlobsRequest) -> Result<(), PdsError> {
    let agent = BskyAgent::builder().build().await.map_err(|error| {
        tracing::error!("{}", error.to_string());
        PdsError::Runtime
    })?;
    let session = login_helper(
        &agent,
        req.origin.as_str(),
        req.did.as_str(),
        req.origin_token.as_str(),
    )
    .await?;
    let blobs = list_all_blobs(&agent).await?;
    let mut path = std::env::current_dir().unwrap();
    path.push(session.did.as_str().replace(":", "-"));
    match tokio::fs::create_dir(path.as_path()).await {
        Ok(_) => {}
        Err(e) => {
            if e.kind() != ErrorKind::AlreadyExists {
                tracing::error!("Error creating directory: {:?}", e);
                return Err(PdsError::Validation);
            }
        }
    }
    for blob in &blobs {
        match get_blob(&agent, blob.clone(), session.did.clone()).await {
            Ok(output) => {
                tracing::info!("Successfully fetched missing blob");
                let mut path = std::env::current_dir().unwrap();
                path.push(session.did.as_str().replace(":", "-"));
                path.push(
                    Cid::to_string(blob.as_ref())
                        .split("/")
                        .last()
                        .unwrap_or("fallback"),
                );
                tokio::fs::write(path.as_path(), output)
                    .await
                    .map_err(|error| {
                        tracing::error!("{}", error.to_string());
                        PdsError::AccountExport
                    })?;
            }
            Err(_) => {
                tracing::error!("Failed to determine missing blobs");
                // return Err(PdsError::Validation);
            }
        }
    }
    Ok(())
}

#[derive(Debug, Deserialize, Serialize)]
pub struct UploadBlobsRequest {
    pub pds_host: String,
    pub did: String,
    pub token: String,
}

#[tracing::instrument]
pub async fn upload_blobs_api(req: UploadBlobsRequest) -> Result<(), PdsError> {
    let agent = BskyAgent::builder().build().await.map_err(|error| {
        tracing::error!("{}", error.to_string());
        PdsError::Runtime
    })?;
    agent.configure_endpoint(req.pds_host.clone());
    let session = login_helper(
        &agent,
        req.pds_host.as_str(),
        req.did.as_str(),
        req.token.as_str(),
    )
    .await?;

    let mut blob_dir;
    let mut path = std::env::current_dir().unwrap();
    path.push(session.did.as_str().replace(":", "-"));
    match tokio::fs::read_dir(path.as_path()).await {
        Ok(output) => blob_dir = output,
        Err(_) => return Err(PdsError::Validation),
    }
    while let Some(blob) = blob_dir.next_entry().await.map_err(|error| {
        tracing::error!("{}", error.to_string());
        PdsError::Runtime
    })? {
        let file = tokio::fs::read(blob.path()).await.map_err(|error| {
            tracing::error!("{}", error.to_string());
            PdsError::Runtime
        })?;
        upload_blob(&agent, file).await?;
    }

    Ok(())
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ActivateAccountRequest {
    pub pds_host: String,
    pub did: String,
    pub token: String,
}

#[tracing::instrument]
pub async fn activate_account_api(req: ActivateAccountRequest) -> Result<(), PdsError> {
    let agent = BskyAgent::builder().build().await.map_err(|error| {
        tracing::error!("{}", error.to_string());
        PdsError::Runtime
    })?;
    login_helper(
        &agent,
        req.pds_host.as_str(),
        req.did.as_str(),
        req.token.as_str(),
    )
    .await?;
    activate_account(&agent).await?;
    Ok(())
}

#[derive(Debug, Deserialize, Serialize)]
pub struct DeactivateAccountRequest {
    pub pds_host: String,
    pub did: String,
    pub token: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct DeactivateAccountResponse {}

#[tracing::instrument]
pub async fn deactivate_account_api(req: DeactivateAccountRequest) -> Result<(), PdsError> {
    let agent = BskyAgent::builder().build().await.map_err(|error| {
        tracing::error!("{}", error.to_string());
        PdsError::Runtime
    })?;
    login_helper(
        &agent,
        req.pds_host.as_str(),
        req.did.as_str(),
        req.token.as_str(),
    )
    .await?;
    deactivate_account(&agent).await?;
    Ok(())
}

#[derive(Debug, Deserialize, Serialize)]
pub struct MigratePreferencesRequest {
    pub destination: String,
    pub destination_token: String,
    pub origin: String,
    pub did: String,
    pub origin_token: String,
}

#[tracing::instrument]
pub async fn migrate_preferences_api(req: MigratePreferencesRequest) -> Result<(), PdsError> {
    let agent = BskyAgent::builder().build().await.map_err(|error| {
        tracing::error!("{}", error.to_string());
        PdsError::Runtime
    })?;
    login_helper(
        &agent,
        req.origin.as_str(),
        req.did.as_str(),
        req.origin_token.as_str(),
    )
    .await?;
    let preferences = export_preferences(&agent).await?;
    login_helper(
        &agent,
        req.destination.as_str(),
        req.did.as_str(),
        req.destination_token.as_str(),
    )
    .await?;
    import_preferences(&agent, preferences).await?;
    Ok(())
}

#[derive(Debug, Deserialize, Serialize)]
pub struct RequestTokenRequest {
    pub pds_host: String,
    pub did: String,
    pub token: String,
}

#[tracing::instrument]
pub async fn request_token_api(req: RequestTokenRequest) -> Result<(), PdsError> {
    let agent = BskyAgent::builder().build().await.map_err(|error| {
        tracing::error!("{}", error.to_string());
        PdsError::Runtime
    })?;
    login_helper(
        &agent,
        req.pds_host.as_str(),
        req.did.as_str(),
        req.token.as_str(),
    )
    .await?;
    request_token(&agent).await?;
    Ok(())
}

#[derive(Debug, Deserialize, Serialize)]
pub struct MigratePlcRequest {
    pub destination: String,
    pub destination_token: String,
    pub origin: String,
    pub did: String,
    pub origin_token: String,
    pub plc_signing_token: String,
    #[serde(skip_serializing_if = "core::option::Option::is_none")]
    pub user_recovery_key: Option<String>,
}

#[tracing::instrument(skip(req))]
pub async fn migrate_plc_api(req: MigratePlcRequest) -> Result<(), PdsError> {
    let agent = BskyAgent::builder().build().await.map_err(|error| {
        tracing::error!("{}", error.to_string());
        PdsError::Runtime
    })?;
    login_helper(
        &agent,
        req.destination.as_str(),
        req.did.as_str(),
        req.destination_token.as_str(),
    )
    .await?;
    let recommended_did = recommended_plc(&agent).await?;
    use bsky_sdk::api::com::atproto::identity::sign_plc_operation::InputData;

    let mut rotation_keys = recommended_did.rotation_keys.unwrap();

    if let Some(recovery_key) = &req.user_recovery_key {
        rotation_keys.insert(0, recovery_key.clone());
    }

    let new_plc = InputData {
        also_known_as: recommended_did.also_known_as,
        rotation_keys: Some(rotation_keys),
        services: recommended_did.services,
        token: Some(req.plc_signing_token.clone()),
        verification_methods: recommended_did.verification_methods,
    };
    login_helper(
        &agent,
        req.origin.as_str(),
        req.did.as_str(),
        req.origin_token.as_str(),
    )
    .await?;
    let output = sign_plc(&agent, new_plc.clone()).await?;
    login_helper(
        &agent,
        req.destination.as_str(),
        req.did.as_str(),
        req.destination_token.as_str(),
    )
    .await?;
    submit_plc(&agent, output).await?;
    Ok(())
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

pub fn public_key_to_did_key(public_key: PublicKey) -> String {
    let pk_compact = public_key.serialize();
    let pk_wrapped = multicodec_wrap(pk_compact.to_vec());
    let pk_multibase = multibase::encode(Base58Btc, pk_wrapped.as_slice());
    let public_key_str = format!("did:key:{pk_multibase}");
    public_key_str
}
