use crate::agent::{
    account_import, activate_account, create_account, deactivate_account, download_blob,
    download_repo, export_preferences, get_service_auth, import_preferences, list_all_blobs,
    login_helper, missing_blobs, recommended_plc, request_token, sign_plc, submit_plc, upload_blob,
};
use crate::errors::PdsError;
use bsky_sdk::api::agent::Configure;
use bsky_sdk::api::types::string::Did;
use bsky_sdk::BskyAgent;
use futures_util::StreamExt;
use multibase::Base::Base58Btc;
use secp256k1::PublicKey;
use serde::{Deserialize, Serialize};
use std::io::ErrorKind;
use std::time::Duration;
use tokio::io::AsyncWriteExt;

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

#[tracing::instrument(skip(req))]
pub async fn get_service_auth_api(req: ServiceAuthRequest) -> Result<String, PdsError> {
    tracing::info!("get_service_auth_api started");
    let agent = BskyAgent::builder().build().await.map_err(|error| {
        tracing::error!("Failed to build BskyAgent: {}", error);
        PdsError::Runtime
    })?;
    login_helper(
        &agent,
        req.pds_host.as_str(),
        req.did.as_str(),
        req.token.as_str(),
    )
    .await?;
    let token = get_service_auth(&agent, req.aud.as_str()).await?;
    tracing::info!("Successfully obtained service auth token");
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
    pub token: Option<String>,
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

#[derive(Debug, Deserialize, Serialize)]
pub struct GetBlobRequest {
    pub did: Did,
    pub cid: String,
    pub token: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct GetRepoRequest {
    pub did: Did,
    pub token: String,
}

#[tracing::instrument(skip(req))]
pub async fn create_account_api(req: CreateAccountApiRequest) -> Result<(), PdsError> {
    tracing::info!("create_account_api started");
    tracing::info!("{:?}", req);

    let did = req.did.parse().map_err(|error| {
        tracing::error!("Failed to parse DID '{}': {}", req.did, error);
        PdsError::Validation
    })?;

    let handle = req.handle.parse().map_err(|error| {
        tracing::error!("Failed to parse handle '{}': {}", req.handle, error);
        PdsError::Validation
    })?;

    create_account(
        req.pds_host.as_str(),
        &CreateAccountRequest {
            did,
            email: Some(req.email.clone()),
            handle,
            invite_code: Some(req.invite_code.trim().to_string()),
            password: Some(req.password.clone()),
            recovery_key: req.recovery_key.clone(),
            verification_code: Some(String::from("")),
            verification_phone: None,
            plc_op: None,
            token: Some(req.token.clone()),
        },
    )
    .await?;
    tracing::info!("create_account_api completed successfully");
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
    let get_repo_request = GetRepoRequest {
        did: session.did.clone(),
        token: session.access_jwt.clone(),
    };
    match download_repo(agent.get_endpoint().await.as_str(), &get_repo_request).await {
        Ok(mut stream) => {
            tracing::info!("Started downloading repo");
            let mut path = std::env::current_dir().map_err(|error| {
                tracing::error!("Failed to get current directory: {}", error);
                PdsError::Runtime
            })?;
            path.push(session.did.clone().replace(":", "-") + ".car");
            tracing::info!("Creating repository file: {}", path.display());

            let mut file = tokio::fs::File::create(path.as_path())
                .await
                .map_err(|error| {
                    tracing::error!("Failed to create file {}: {}", path.display(), error);
                    PdsError::Runtime
                })?;

            while let Some(chunk) = stream.next().await {
                let chunk = chunk.map_err(|error| {
                    tracing::error!("Error reading stream chunk: {:?}", error);
                    PdsError::Runtime
                })?;
                file.write_all(&chunk).await.map_err(|error| {
                    tracing::error!("Failed to write chunk to file: {}", error);
                    PdsError::Runtime
                })?;
            }
            file.flush().await.map_err(|error| {
                tracing::error!("Failed to flush file: {}", error);
                PdsError::Runtime
            })?;
            tracing::info!("Successfully exported repository to {}", path.display());
            return Ok(());
        }
        Err(e) => {
            match e {
                PdsError::RateLimitReached => {
                    tracing::error!("Rate limit reached, waiting 5 minutes");
                    let five_minutes = Duration::from_secs(300);
                    tokio::time::sleep(five_minutes).await;
                }
                _ => {
                    tracing::error!("Failed to download repo");
                    return Err(PdsError::Validation);
                }
            }
            tracing::error!("Failed to download Repo");
        }
    }
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
    tracing::info!("missing_blobs_api started");
    let agent = BskyAgent::builder().build().await.map_err(|error| {
        tracing::error!("Failed to build BskyAgent: {}", error);
        PdsError::Runtime
    })?;
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
        missing_blob_cids.push(format!("{:?}", blob.cid));
    }
    tracing::info!("Found {} missing blobs", missing_blob_cids.len());

    let response = serde_json::to_string(&missing_blob_cids).map_err(|error| {
        tracing::error!("Failed to serialize missing blob CIDs: {}", error);
        PdsError::Runtime
    })?;
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
    tracing::info!("export_blobs_api started");
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
    let mut path = match std::env::current_dir() {
        Ok(path) => path,
        Err(e) => {
            tracing::error!("Error getting current directory: {:?}", e);
            return Err(PdsError::Validation);
        }
    };
    path.push(session.did.as_str().replace(":", "-"));
    match tokio::fs::create_dir(path.as_path()).await {
        Ok(_) => {
            tracing::info!("Successfully created directory");
        }
        Err(e) => {
            if e.kind() != ErrorKind::AlreadyExists {
                tracing::error!("Error creating directory: {:?}", e);
                return Err(PdsError::Validation);
            }
        }
    }
    for missing_blob in &missing_blobs {
        tracing::debug!("Missing blob: {:?}", missing_blob);
        let session = match agent.get_session().await {
            Some(session) => session,
            None => {
                tracing::error!("Failed to get session");
                return Err(PdsError::Validation);
            }
        };
        let mut filepath = match std::env::current_dir() {
            Ok(res) => res,
            Err(e) => {
                tracing::error!("Error getting current directory: {:?}", e);
                return Err(PdsError::Validation);
            }
        };
        filepath.push(session.did.as_str().replace(":", "-"));
        filepath.push(
            missing_blob
                .record_uri
                .as_str()
                .split("/")
                .last()
                .unwrap_or("fallback"),
        );
        if !tokio::fs::try_exists(filepath).await.unwrap() {
            let missing_blob_cid = missing_blob.cid.clone();
            let get_blob_request = GetBlobRequest {
                did: session.did.clone(),
                cid: format!("{missing_blob_cid:?}")
                    .strip_prefix("Cid(Cid(")
                    .unwrap()
                    .strip_suffix("))")
                    .unwrap()
                    .to_string(),
                token: session.access_jwt.clone(),
            };
            match download_blob(agent.get_endpoint().await.as_str(), &get_blob_request).await {
                Ok(mut stream) => {
                    tracing::info!("Successfully fetched missing blob");
                    let mut path = std::env::current_dir().unwrap();
                    path.push(session.did.as_str().replace(":", "-"));
                    path.push(
                        format!("{missing_blob_cid:?}")
                            .strip_prefix("Cid(Cid(")
                            .unwrap()
                            .strip_suffix("))")
                            .unwrap(),
                    );
                    let mut file = tokio::fs::File::create(path.as_path()).await.unwrap();

                    while let Some(chunk) = stream.next().await {
                        let chunk = chunk.unwrap();
                        file.write_all(&chunk).await.unwrap();
                    }

                    file.flush().await.unwrap();
                }
                Err(e) => {
                    match e {
                        PdsError::RateLimitReached => {
                            tracing::error!("Rate limit reached, waiting 5 minutes");
                            let five_minutes = Duration::from_secs(300);
                            tokio::time::sleep(five_minutes).await;
                        }
                        PdsError::Validation => {}
                        _ => {
                            tracing::error!("Failed to determine missing blobs");
                            return Err(PdsError::Validation);
                        }
                    }
                    tracing::error!("Failed to determine missing blobs");
                }
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
        let session = agent.get_session().await.unwrap();
        let mut filepath = std::env::current_dir().unwrap();
        filepath.push(session.did.as_str().replace(":", "-"));
        filepath.push(
            format!("{blob:?}")
                .strip_prefix("Cid(Cid(")
                .unwrap()
                .strip_suffix("))")
                .unwrap(),
        );
        if !tokio::fs::try_exists(filepath).await.unwrap() {
            let get_blob_request = GetBlobRequest {
                did: session.did.clone(),
                cid: format!("{blob:?}")
                    .strip_prefix("Cid(Cid(")
                    .unwrap()
                    .strip_suffix("))")
                    .unwrap()
                    .to_string(),
                token: session.access_jwt.clone(),
            };
            match download_blob(agent.get_endpoint().await.as_str(), &get_blob_request).await {
                Ok(mut stream) => {
                    tracing::info!("Successfully fetched missing blob");
                    let mut path = std::env::current_dir().unwrap();
                    path.push(session.did.as_str().replace(":", "-"));
                    path.push(
                        format!("{blob:?}")
                            .strip_prefix("Cid(Cid(")
                            .unwrap()
                            .strip_suffix("))")
                            .unwrap(),
                    );
                    let mut file = tokio::fs::File::create(path.as_path()).await.unwrap();

                    while let Some(chunk) = stream.next().await {
                        let chunk = chunk.unwrap();
                        file.write_all(&chunk).await.unwrap();
                    }

                    file.flush().await.unwrap();
                }
                Err(e) => {
                    match e {
                        PdsError::RateLimitReached => {
                            tracing::error!("Rate limit reached, waiting 5 minutes");
                            let five_minutes = Duration::from_secs(300);
                            tokio::time::sleep(five_minutes).await;
                        }
                        _ => {
                            tracing::error!("Failed to determine missing blobs");
                            return Err(PdsError::Validation);
                        }
                    }
                    tracing::error!("Failed to determine missing blobs");
                }
            }
        }
    }
    //

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

#[cfg(test)]
mod tests {
    use super::*;
    use secp256k1::{Secp256k1, SecretKey};
    use std::str::FromStr;

    #[test]
    fn test_multicodec_wrap() {
        let test_bytes = vec![0x01, 0x02, 0x03];
        let wrapped = multicodec_wrap(test_bytes.clone());

        // Should start with secp256k1 multicodec prefix (0xe7)
        assert_eq!(wrapped[0], 0xe7);
        // Should contain original bytes at the end
        assert!(wrapped.ends_with(&test_bytes));
        assert!(wrapped.len() > test_bytes.len());
    }

    #[test]
    fn test_public_key_to_did_key() {
        let secp = Secp256k1::new();
        // Use a known test private key
        let secret_key =
            SecretKey::from_str("0000000000000000000000000000000000000000000000000000000000000001")
                .expect("Valid secret key");
        let public_key = PublicKey::from_secret_key(&secp, &secret_key);

        let did_key = public_key_to_did_key(public_key);

        // Should start with "did:key:"
        assert!(did_key.starts_with("did:key:"));
        // Should be a reasonable length (did:key: + base58btc encoded multicodec wrapped pubkey)
        assert!(did_key.len() > 50);
        // Should be deterministic for the same key
        let did_key_2 = public_key_to_did_key(public_key);
        assert_eq!(did_key, did_key_2);
    }

    #[test]
    fn test_multicodec_wrap_empty() {
        let empty_bytes = vec![];
        let wrapped = multicodec_wrap(empty_bytes);

        // Should still have the multicodec prefix even for empty input
        assert_eq!(wrapped[0], 0xe7);
        // Varint encoding of 0xe7 takes 2 bytes since 0xe7 > 127
        assert_eq!(wrapped.len(), 2);
    }

    #[test]
    fn test_service_auth_request_serialization() {
        let request = ServiceAuthRequest {
            pds_host: "https://example.com".to_string(),
            aud: "https://audience.com".to_string(),
            did: "did:plc:test123".to_string(),
            token: "auth_token_123".to_string(),
        };

        let json = serde_json::to_string(&request).unwrap();
        assert!(json.contains("https://example.com"));
        assert!(json.contains("https://audience.com"));
        assert!(json.contains("did:plc:test123"));
        assert!(json.contains("auth_token_123"));

        let deserialized: ServiceAuthRequest = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.pds_host, "https://example.com");
        assert_eq!(deserialized.aud, "https://audience.com");
        assert_eq!(deserialized.did, "did:plc:test123");
        assert_eq!(deserialized.token, "auth_token_123");
    }

    #[test]
    fn test_service_auth_response_serialization() {
        let response = ServiceAuthResponse {
            token: "response_token_456".to_string(),
        };

        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("response_token_456"));

        let deserialized: ServiceAuthResponse = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.token, "response_token_456");
    }

    #[test]
    fn test_create_account_api_request_serialization() {
        let request = CreateAccountApiRequest {
            email: "test@example.com".to_string(),
            handle: "test.handle".to_string(),
            invite_code: "invite123".to_string(),
            password: "password123".to_string(),
            token: "token123".to_string(),
            pds_host: "https://pds.example.com".to_string(),
            did: "did:plc:test123".to_string(),
            recovery_key: Some("recovery_key_123".to_string()),
        };

        let json = serde_json::to_string(&request).unwrap();
        assert!(json.contains("test@example.com"));
        assert!(json.contains("test.handle"));
        assert!(json.contains("https://pds.example.com"));

        let deserialized: CreateAccountApiRequest = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.email, "test@example.com");
        assert_eq!(deserialized.handle, "test.handle");
        assert_eq!(
            deserialized.recovery_key,
            Some("recovery_key_123".to_string())
        );
    }

    #[test]
    fn test_get_blob_request_serialization() {
        use bsky_sdk::api::types::string::Did;

        let did_str = "did:plc:test123";
        let did: Did = did_str.parse().unwrap();

        let request = GetBlobRequest {
            did: did.clone(),
            cid: "bafytest123".to_string(),
            token: "blob_token_789".to_string(),
        };

        let json = serde_json::to_string(&request).unwrap();
        assert!(json.contains("bafytest123"));
        assert!(json.contains("blob_token_789"));

        let deserialized: GetBlobRequest = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.did, did);
        assert_eq!(deserialized.cid, "bafytest123");
        assert_eq!(deserialized.token, "blob_token_789");
    }

    #[test]
    fn test_get_repo_request_serialization() {
        use bsky_sdk::api::types::string::Did;

        let did_str = "did:plc:test123";
        let did: Did = did_str.parse().unwrap();

        let request = GetRepoRequest {
            did: did.clone(),
            token: "repo_token_101".to_string(),
        };

        let json = serde_json::to_string(&request).unwrap();
        assert!(json.contains("repo_token_101"));

        let deserialized: GetRepoRequest = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.did, did);
        assert_eq!(deserialized.token, "repo_token_101");
    }

    #[test]
    fn test_export_pds_request_serialization() {
        let request = ExportPDSRequest {
            pds_host: "https://origin.example.com".to_string(),
            did: "did:plc:test123".to_string(),
            token: "export_token_202".to_string(),
        };

        let json = serde_json::to_string(&request).unwrap();
        assert!(json.contains("https://origin.example.com"));
        assert!(json.contains("did:plc:test123"));
        assert!(json.contains("export_token_202"));

        let deserialized: ExportPDSRequest = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.pds_host, "https://origin.example.com");
        assert_eq!(deserialized.did, "did:plc:test123");
        assert_eq!(deserialized.token, "export_token_202");
    }

    #[test]
    fn test_multicodec_wrap_edge_cases() {
        // Test with single byte
        let single_byte = vec![0x42];
        let wrapped = multicodec_wrap(single_byte.clone());
        assert_eq!(wrapped[0], 0xe7);
        assert!(wrapped.ends_with(&single_byte));
        assert_eq!(wrapped.len(), 3); // 2 bytes for varint + 1 byte input

        // Test with max value bytes
        let max_bytes = vec![0xFF, 0xFF, 0xFF];
        let wrapped = multicodec_wrap(max_bytes.clone());
        assert_eq!(wrapped[0], 0xe7);
        assert!(wrapped.ends_with(&max_bytes));
    }

    #[test]
    fn test_public_key_to_did_key_format() {
        let secp = Secp256k1::new();
        let secret_key =
            SecretKey::from_str("0000000000000000000000000000000000000000000000000000000000000001")
                .expect("Valid secret key");
        let public_key = PublicKey::from_secret_key(&secp, &secret_key);

        let did_key = public_key_to_did_key(public_key);

        // Verify DID key format
        assert!(did_key.starts_with("did:key:z"));

        // Verify it's base58btc encoded (should contain valid base58btc characters)
        let encoded_part = &did_key[8..]; // Skip "did:key:" prefix
        assert!(encoded_part
            .chars()
            .all(|c| "123456789ABCDEFGHJKLMNPQRSTUVWXYZabcdefghijkmnopqrstuvwxyz".contains(c)));

        // Verify reasonable length (multicodec + compressed pubkey + base58 overhead)
        assert!(did_key.len() >= 50 && did_key.len() <= 100);
    }

    #[test]
    fn test_create_account_request_optional_fields() {
        use bsky_sdk::api::types::string::Did;

        let did: Did = "did:plc:test123".parse().unwrap();

        // Test with minimal fields
        let minimal_request = CreateAccountRequest {
            did: did.clone(),
            email: None,
            handle: "test.handle".to_string(),
            invite_code: None,
            password: None,
            recovery_key: None,
            verification_code: None,
            verification_phone: None,
            plc_op: None,
            token: None,
        };

        let json = serde_json::to_string(&minimal_request).unwrap();
        let deserialized: CreateAccountRequest = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.did, did);
        assert_eq!(deserialized.handle, "test.handle");
        assert!(deserialized.email.is_none());
        assert!(deserialized.password.is_none());

        // Test with all fields
        let full_request = CreateAccountRequest {
            did: did.clone(),
            email: Some("test@example.com".to_string()),
            handle: "test.handle".to_string(),
            invite_code: Some("invite123".to_string()),
            password: Some("password123".to_string()),
            recovery_key: Some("recovery123".to_string()),
            verification_code: Some("verify123".to_string()),
            verification_phone: Some("+1234567890".to_string()),
            plc_op: Some("plc_op_data".to_string()),
            token: Some("token123".to_string()),
        };

        let json = serde_json::to_string(&full_request).unwrap();
        let deserialized: CreateAccountRequest = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.email, Some("test@example.com".to_string()));
        assert_eq!(
            deserialized.verification_phone,
            Some("+1234567890".to_string())
        );
    }
}
