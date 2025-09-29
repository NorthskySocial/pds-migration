use crate::errors::PdsError;
use crate::{CreateAccountRequest, CreateAccountWithoutPDSRequest, GetBlobRequest, GetRepoRequest};
use bsky_sdk::api::agent::atp_agent::AtpSession;
use bsky_sdk::api::agent::Configure;
use bsky_sdk::api::app::bsky::actor::defs::Preferences;
use bsky_sdk::api::com::atproto::identity::sign_plc_operation::InputData;
use bsky_sdk::api::com::atproto::repo::list_missing_blobs::RecordBlob;
use bsky_sdk::api::types::string::{Cid, Did, Handle, Nsid};
use bsky_sdk::api::types::Unknown;
use bsky_sdk::BskyAgent;
use ipld_core::ipld::Ipld;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

const PLC_DIRECTORY: &str = "https://plc.directory";
pub type GetAgentResult = Result<BskyAgent, Box<dyn std::error::Error>>;
pub type RecommendedDidOutputData =
    bsky_sdk::api::com::atproto::identity::get_recommended_did_credentials::OutputData;

#[tracing::instrument(skip(agent, token))]
pub async fn login_helper(
    agent: &BskyAgent,
    pds_host: &str,
    did: &str,
    token: &str,
) -> Result<AtpSession, PdsError> {
    use bsky_sdk::api::com::atproto::server::create_session::OutputData;
    agent.configure_endpoint(pds_host.to_string());
    match agent
        .resume_session(AtpSession {
            data: OutputData {
                access_jwt: token.to_string(),
                active: Some(true),
                did: Did::new(did.to_string()).unwrap(),
                did_doc: None,
                email: None,
                email_auth_factor: None,
                email_confirmed: None,
                handle: Handle::new("anothermigration.bsky.social".to_string()).unwrap(),
                refresh_jwt: "".to_string(),
                status: None,
            },
            extra_data: Ipld::Null,
        })
        .await
    {
        Ok(_) => {
            tracing::info!("Successfully logged in");
            Ok(agent.get_session().await.unwrap())
        }
        Err(e) => {
            tracing::error!("Error logging in: {:?}", e);
            Err(PdsError::Login)
        }
    }
}

#[tracing::instrument(skip(agent))]
pub async fn list_all_blobs(agent: &BskyAgent) -> Result<Vec<Cid>, PdsError> {
    use bsky_sdk::api::com::atproto::sync::list_blobs::{Parameters, ParametersData};
    let mut result = vec![];
    let mut cursor = None;
    let mut length = None;
    let did = agent.did().await.clone().unwrap();
    while length.is_none() || length.unwrap() >= 500 {
        let output = agent
            .api
            .com
            .atproto
            .sync
            .list_blobs(Parameters {
                data: ParametersData {
                    cursor: cursor.clone(),
                    did: did.clone(),
                    limit: None,
                    since: None,
                },
                extra_data: Ipld::Null,
            })
            .await;
        match output {
            Ok(output) => {
                tracing::info!("{:?}", output);
                cursor = output.cursor.clone();
                length = Some(output.cids.len());
                let mut blob_cids = output.cids.clone();
                result.append(blob_cids.as_mut());
            }
            Err(e) => {
                tracing::error!("{:?}", e);
                return Err(PdsError::Validation);
            }
        }
    }
    Ok(result)
}

#[tracing::instrument(skip(agent))]
pub async fn missing_blobs(agent: &BskyAgent) -> Result<Vec<RecordBlob>, PdsError> {
    use bsky_sdk::api::com::atproto::repo::list_missing_blobs::{Parameters, ParametersData};
    let mut result: Vec<RecordBlob> = vec![];
    let mut length = None;
    let mut cursor = None;
    while length.is_none() || length.unwrap() >= 500 {
        let output = agent
            .api
            .com
            .atproto
            .repo
            .list_missing_blobs(Parameters {
                data: ParametersData {
                    cursor: cursor.clone(),
                    limit: None,
                },
                extra_data: Ipld::Null,
            })
            .await;
        match output {
            Ok(output) => {
                tracing::info!("{:?}", output);
                length = Some(output.blobs.len());
                let mut temp = output.blobs.clone();
                result.append(temp.as_mut());
                cursor = output.cursor.clone();
            }
            Err(e) => {
                tracing::error!("{:?}", e);
                return Err(PdsError::Validation);
            }
        }
    }
    Ok(result)
}

#[tracing::instrument(skip(agent))]
pub async fn get_blob(agent: &BskyAgent, cid: Cid, did: Did) -> Result<Vec<u8>, ()> {
    use bsky_sdk::api::com::atproto::sync::get_blob::{Parameters, ParametersData};
    let result = agent
        .api
        .com
        .atproto
        .sync
        .get_blob(Parameters {
            data: ParametersData {
                cid,
                did: did.parse().unwrap(),
            },
            extra_data: Ipld::Null,
        })
        .await;
    match result {
        Ok(output) => {
            tracing::debug!("Successfully fetched blob: {:?}", output);
            Ok(output.clone())
        }
        Err(e) => {
            tracing::error!("Failed to fetch blob: {:?}", e);
            Err(())
        }
    }
}

#[tracing::instrument(skip(agent, input))]
pub async fn upload_blob(agent: &BskyAgent, input: Vec<u8>) -> Result<(), PdsError> {
    let result = agent.api.com.atproto.repo.upload_blob(input).await;
    match result {
        Ok(output) => {
            tracing::info!("Successfully uploaded blob");
            tracing::debug!("{:?}", output);
            Ok(())
        }
        Err(e) => {
            tracing::error!("Failed to upload blob: {:?}", e);
            Err(PdsError::Validation)
        }
    }
}

#[tracing::instrument(skip(agent))]
pub async fn export_preferences(agent: &BskyAgent) -> Result<Preferences, PdsError> {
    use bsky_sdk::api::app::bsky::actor::get_preferences::{Parameters, ParametersData};
    let result = agent
        .api
        .app
        .bsky
        .actor
        .get_preferences(Parameters {
            data: ParametersData {},
            extra_data: Ipld::Null,
        })
        .await;
    match result {
        Ok(output) => {
            tracing::info!("Successfully exported preferences");
            tracing::debug!("{:?}", output);
            Ok(output.preferences.clone())
        }
        Err(e) => {
            tracing::error!("Failed to export preferences: {:?}", e);
            Err(PdsError::Validation)
        }
    }
}

#[tracing::instrument(skip(agent))]
pub async fn import_preferences(
    agent: &BskyAgent,
    preferences: Preferences,
) -> Result<(), PdsError> {
    use bsky_sdk::api::app::bsky::actor::put_preferences::{Input, InputData};
    let result = agent
        .api
        .app
        .bsky
        .actor
        .put_preferences(Input {
            data: InputData { preferences },
            extra_data: Ipld::Null,
        })
        .await;
    match result {
        Ok(output) => {
            tracing::info!("Successfully imported preferences");
            tracing::debug!("{:?}", output);
            Ok(())
        }
        Err(e) => {
            tracing::error!("Failed to import preferences: {:?}", e);
            Err(PdsError::Validation)
        }
    }
}

#[tracing::instrument(skip(agent))]
pub async fn recommended_plc(agent: &BskyAgent) -> Result<RecommendedDidOutputData, PdsError> {
    let result = agent
        .api
        .com
        .atproto
        .identity
        .get_recommended_did_credentials()
        .await;
    match result {
        Ok(output) => {
            tracing::info!("Successfully imported preferences");
            tracing::debug!("{:?}", output);
            Ok(output.data)
        }
        Err(e) => {
            tracing::error!("Failed to import preferences: {:?}", e);
            Err(PdsError::Validation)
        }
    }
}

#[tracing::instrument(skip(agent))]
pub async fn get_service_auth(agent: &BskyAgent, aud: &str) -> Result<String, PdsError> {
    use bsky_sdk::api::com::atproto::server::get_service_auth::{Parameters, ParametersData};
    let result = agent
        .api
        .com
        .atproto
        .server
        .get_service_auth(Parameters {
            data: ParametersData {
                aud: aud.parse().unwrap(),
                exp: None,
                lxm: Some(Nsid::new("com.atproto.server.createAccount".to_string()).unwrap()),
            },
            extra_data: Ipld::Null,
        })
        .await;
    match result {
        Ok(output) => {
            tracing::info!("Successfully requested service auth");
            tracing::debug!("{:?}", output);
            Ok(output.token.clone())
        }
        Err(e) => {
            tracing::error!("Failed to request service auth: {:?}", e);
            Err(PdsError::Runtime)
        }
    }
}

#[tracing::instrument(skip(agent))]
pub async fn sign_plc(agent: &BskyAgent, plc_input_data: InputData) -> Result<Unknown, PdsError> {
    use bsky_sdk::api::com::atproto::identity::sign_plc_operation::Input;
    let result = agent
        .api
        .com
        .atproto
        .identity
        .sign_plc_operation(Input {
            data: plc_input_data,
            extra_data: Ipld::Null,
        })
        .await;
    match result {
        Ok(output) => {
            tracing::info!("Successfully signed token");
            tracing::debug!("{:?}", output);
            Ok(output.operation.clone())
        }
        Err(e) => {
            tracing::error!("Failed to sign token: {:?}", e);
            Err(PdsError::Validation)
        }
    }
}

#[tracing::instrument(skip(agent))]
pub async fn account_import(agent: &BskyAgent, filepath: &str) -> Result<(), PdsError> {
    let result = agent
        .api
        .com
        .atproto
        .repo
        .import_repo(tokio::fs::read(filepath).await.unwrap())
        .await;
    match result {
        Ok(_) => {
            tracing::info!("Successfully imported account");
            Ok(())
        }
        Err(e) => {
            tracing::error!("Error importing: {:?}", e.to_string());
            Err(PdsError::AccountImport)
        }
    }
}

#[tracing::instrument(skip(agent))]
pub async fn account_export(agent: &BskyAgent, did: &Did) -> Result<(), PdsError> {
    use bsky_sdk::api::com::atproto::sync::get_repo::{Parameters, ParametersData};
    let result = agent
        .api
        .com
        .atproto
        .sync
        .get_repo(Parameters {
            data: ParametersData {
                did: did.clone(),
                since: None,
            },
            extra_data: Ipld::Null,
        })
        .await;
    match result {
        Ok(output) => {
            tokio::fs::write(did.as_str().to_string().replace(":", "-") + ".car", output)
                .await
                .map_err(|error| {
                    tracing::error!("{}", error.to_string());
                    PdsError::AccountExport
                })?;
            tracing::info!("write success");
            Ok(())
        }
        Err(e) => {
            tracing::error!("Error exporting: {:?}", e);
            Err(PdsError::AccountExport)
        }
    }
}

#[tracing::instrument(skip(agent))]
pub async fn deactivate_account(agent: &BskyAgent) -> Result<(), PdsError> {
    use bsky_sdk::api::com::atproto::server::deactivate_account::{Input, InputData};
    let result = agent
        .api
        .com
        .atproto
        .server
        .deactivate_account(Input {
            data: InputData { delete_after: None },
            extra_data: Ipld::Null,
        })
        .await;
    match result {
        Ok(output) => {
            tracing::info!("Successfully deactivated account");
            tracing::debug!("{:?}", output);
            Ok(())
        }
        Err(e) => {
            tracing::error!("Failed to deactivate account: {:?}", e);
            Err(PdsError::Validation)
        }
    }
}

#[tracing::instrument(skip(agent))]
pub async fn activate_account(agent: &BskyAgent) -> Result<(), PdsError> {
    let result = agent.api.com.atproto.server.activate_account().await;
    match result {
        Ok(output) => {
            tracing::info!("Successfully activated account");
            tracing::debug!("{:?}", output);
            Ok(())
        }
        Err(e) => {
            tracing::error!("Failed to activate account: {:?}", e);
            Err(PdsError::Validation)
        }
    }
}

#[tracing::instrument(skip(agent))]
pub async fn submit_plc(agent: &BskyAgent, signed_plc: Unknown) -> Result<(), PdsError> {
    use bsky_sdk::api::com::atproto::identity::submit_plc_operation::{Input, InputData};
    let result = agent
        .api
        .com
        .atproto
        .identity
        .submit_plc_operation(Input {
            data: InputData {
                operation: signed_plc,
            },
            extra_data: Ipld::Null,
        })
        .await;
    match result {
        Ok(output) => {
            tracing::info!("Successfully submitted PLC Operation");
            tracing::debug!("{:?}", output);
            Ok(())
        }
        Err(e) => {
            tracing::error!("Failed to submitted PLC Operation: {:?}", e);
            Err(PdsError::Validation)
        }
    }
}

#[tracing::instrument(skip(agent))]
pub async fn request_token(agent: &BskyAgent) -> Result<(), PdsError> {
    let result = agent
        .api
        .com
        .atproto
        .identity
        .request_plc_operation_signature()
        .await;
    match result {
        Ok(_) => Ok(()),
        Err(e) => {
            tracing::error!("{:?}", e);
            Err(PdsError::Validation)
        }
    }
}

#[tracing::instrument(skip(account_request))]
pub async fn create_account(
    pds_host: &str,
    account_request: &CreateAccountRequest,
) -> Result<(), PdsError> {
    use bsky_sdk::api::com::atproto::server::create_account::{Input, InputData};
    let client = reqwest::Client::new();
    let x = serde_json::to_string(&Input {
        data: InputData {
            did: Some(account_request.did.clone()),
            email: account_request.email.clone(),
            handle: account_request.handle.parse().unwrap(),
            invite_code: account_request.invite_code.clone(),
            password: account_request.password.clone(),
            plc_op: None,
            recovery_key: account_request.recovery_key.clone(),
            verification_code: account_request.verification_code.clone(),
            verification_phone: account_request.verification_phone.clone(),
        },
        extra_data: Ipld::Null,
    })
    .unwrap();
    let result = client
        .post(pds_host.to_string() + "/xrpc/com.atproto.server.createAccount")
        .body(x)
        .header("Content-Type", "application/json")
        .bearer_auth(account_request.token.clone())
        .send()
        .await;
    match result {
        Ok(output) => match output.status() {
            reqwest::StatusCode::OK => {
                tracing::info!("Successfully created account");
            }
            _ => {
                tracing::error!("Error creating account: {:?}", output);
                tracing::error!("More: {:?}", output.text().await);
                return Err(PdsError::Validation);
            }
        },
        Err(e) => {
            tracing::error!("Error creating account: {:?}", e);
            return Err(PdsError::Validation);
        }
    }
    Ok(())
}

#[tracing::instrument(skip(request))]
pub async fn download_blob(
    pds_host: &str,
    request: &GetBlobRequest,
) -> Result<impl futures_core::Stream<Item = Result<bytes::Bytes, reqwest::Error>>, PdsError> {
    let client = reqwest::Client::new();
    let url = format!("{pds_host}/xrpc/com.atproto.sync.getBlob");
    let result = client
        .get(url)
        .query(&[
            ("did", request.did.as_str().to_string()),
            ("cid", request.cid.clone()),
        ])
        .header("Content-Type", "application/json")
        .bearer_auth(request.token.clone())
        .send()
        .await;
    match result {
        Ok(output) => {
            let ratelimit_remaining = output
                .headers()
                .get("ratelimit-remaining")
                .unwrap()
                .to_str()
                .unwrap_or("1000")
                .parse::<i32>()
                .unwrap_or(1000);
            if ratelimit_remaining < 100 {
                return Err(PdsError::RateLimitReached);
            }

            match output.status() {
                reqwest::StatusCode::OK => {
                    tracing::info!("Successfully downloaded blob");
                    Ok(output.bytes_stream())
                }
                _ => {
                    tracing::error!("Error downloading blob: {:?}", output);
                    Err(PdsError::Validation)
                }
            }
        }
        Err(e) => {
            tracing::error!("Error downloading blob: {:?}", e);
            Err(PdsError::Validation)
        }
    }
}

#[tracing::instrument(skip(request))]
pub async fn download_repo(
    pds_host: &str,
    request: &GetRepoRequest,
) -> Result<impl futures_core::Stream<Item = Result<bytes::Bytes, reqwest::Error>>, PdsError> {
    let client = reqwest::Client::new();

    let url = format!("{pds_host}/xrpc/com.atproto.sync.getRepo");
    let result = client
        .get(url)
        .query(&[("did", request.did.as_str().to_string())])
        .send()
        .await;
    match result {
        Ok(output) => {
            let ratelimit_remaining = match output.headers().get("ratelimit-remaining") {
                None => 1000,
                Some(rate_limit_remaining) => rate_limit_remaining
                    .to_str()
                    .unwrap_or("1000")
                    .parse::<i32>()
                    .unwrap_or(1000),
            };
            if ratelimit_remaining < 100 {
                tracing::error!("Ratelimit reached");
                return Err(PdsError::RateLimitReached);
            }

            match output.status() {
                reqwest::StatusCode::OK => {
                    tracing::info!("Started downloading Repo");
                    Ok(output.bytes_stream())
                }
                _ => {
                    tracing::error!("Error downloading Repo: {:?}", output);
                    Err(PdsError::Validation)
                }
            }
        }
        Err(e) => {
            tracing::error!("Error download Repo: {:?}", e);
            Err(PdsError::Validation)
        }
    }
}

#[tracing::instrument(skip(account_request))]
pub async fn create_account_without_pds(
    pds_host: &str,
    account_request: &CreateAccountWithoutPDSRequest,
) -> Result<(), PdsError> {
    use bsky_sdk::api::com::atproto::server::create_account::{Input, InputData};
    let client = reqwest::Client::new();
    let x = serde_json::to_string(&Input {
        data: InputData {
            did: Some(account_request.did.clone()),
            email: account_request.email.clone(),
            handle: account_request.handle.parse().unwrap(),
            invite_code: account_request.invite_code.clone(),
            password: account_request.password.clone(),
            plc_op: None,
            recovery_key: account_request.recovery_key.clone(),
            verification_code: account_request.verification_code.clone(),
            verification_phone: account_request.verification_phone.clone(),
        },
        extra_data: Ipld::Null,
    })
    .unwrap();
    let result = client
        .post(pds_host.to_string() + "/xrpc/com.atproto.server.createAccount")
        .body(x)
        .header("Content-Type", "application/json")
        .send()
        .await;
    match result {
        Ok(output) => match output.status() {
            reqwest::StatusCode::OK => {
                tracing::info!("Successfully created account");
            }
            _ => {
                tracing::error!("Error creating account: {:?}", output);
                tracing::error!("More: {:?}", output.text().await);
                return Err(PdsError::Validation);
            }
        },
        Err(e) => {
            tracing::error!("Error creating account: {:?}", e);
            return Err(PdsError::Validation);
        }
    }
    Ok(())
}

pub type PlcLogAudit = Vec<PlcLogAuditEntry>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlcLogAuditEntry {
    pub did: String,
    pub operation: PlcOperation,
    pub cid: String,
    pub nullified: bool,
    #[serde(rename = "createdAt")]
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlcOperation {
    #[serde(rename = "type")]
    pub r#type: String,
    #[serde(rename = "rotationKeys")]
    pub rotation_keys: Vec<String>,
    #[serde(rename = "verificationMethods")]
    pub verification_methods: BTreeMap<String, String>,
    #[serde(rename = "alsoKnownAs")]
    pub also_known_as: Vec<String>,
    pub services: BTreeMap<String, PlcOpService>,
    pub prev: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sig: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlcOpService {
    #[serde(rename = "type")]
    pub r#type: String,
    pub endpoint: String,
}

#[tracing::instrument]
pub async fn get_plc_audit_log(did: &str) -> PlcLogAudit {
    let client = reqwest::Client::new();
    let plc_audit = match client
        .get(PLC_DIRECTORY.to_string() + format!("/{did}/log/audit").as_str())
        .send()
        .await
    {
        Ok(result) => match result.json::<PlcLogAudit>().await {
            Ok(res) => res,
            Err(e) => {
                panic!("Error: Could not parse response {e}");
            }
        },
        Err(e) => {
            panic!("Error: {e:?}");
        }
    };
    plc_audit
}

pub async fn generate_service_auth_without_pds() {}

#[tracing::instrument]
pub async fn send_plc_operation(did: &str, op: PlcOperation) {
    let client = reqwest::Client::new();
    match client
        .post(PLC_DIRECTORY.to_string() + format!("/{did}").as_str())
        .header("Content-Type", "application/json")
        .json(&op)
        .send()
        .await
    {
        Ok(result) => {
            println!("{result:?}");
            println!("{:?}", result.status());
            println!("{:?}", result.text().await);
            // println!("{:?}", result.json());
        }
        Err(e) => {
            panic!("Error: {e:?}");
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetRecommendedResponse {
    #[serde(rename = "rotationKeys")]
    pub rotation_keys: Vec<String>,
    #[serde(rename = "alsoKnownAs")]
    pub also_known_as: Vec<String>,
    pub services: BTreeMap<String, PlcOpService>,
    #[serde(rename = "verificationMethods")]
    pub verification_methods: BTreeMap<String, String>,
}

#[tracing::instrument(skip(access_token))]
pub async fn get_recommended(pds_host: &str, access_token: &str) -> GetRecommendedResponse {
    let client = reqwest::Client::new();
    let result = client
        .get(pds_host.to_string() + "/xrpc/com.atproto.identity.getRecommendedDidCredentials")
        .bearer_auth(access_token)
        .send()
        .await;
    match result {
        Ok(output) => match output.status() {
            reqwest::StatusCode::OK => {
                tracing::info!("Successfully Fetched Recommended account");
                output.json::<GetRecommendedResponse>().await.unwrap()
            }
            _ => {
                tracing::error!("Error fetching recommended account: {:?}", output);
                panic!("Error: {:?}", output.text().await);
            }
        },
        Err(e) => {
            tracing::error!("Error fetching recommended: {:?}", e);
            panic!("Error: {e:?}");
        }
    }
}
