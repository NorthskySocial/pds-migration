use crate::errors::PdsError;
use crate::CreateAccountRequest;
use bsky_sdk::api::agent::atp_agent::AtpSession;
use bsky_sdk::api::agent::Configure;
use bsky_sdk::api::app::bsky::actor::defs::Preferences;
use bsky_sdk::api::com::atproto::identity::sign_plc_operation::InputData;
use bsky_sdk::api::com::atproto::repo::list_missing_blobs::RecordBlob;
use bsky_sdk::api::types::string::{Cid, Did, Handle, Nsid};
use bsky_sdk::api::types::Unknown;
use bsky_sdk::BskyAgent;
use ipld_core::ipld::Ipld;

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
pub async fn describe_server(
    agent: &BskyAgent,
) -> Result<bsky_sdk::api::com::atproto::server::describe_server::OutputData, String> {
    let result = agent.api.com.atproto.server.describe_server().await;
    match result {
        Ok(output) => {
            tracing::info!("{:?}", output);
            Ok(output.data)
        }
        Err(e) => {
            tracing::error!("{:?}", e);
            Err(String::from("Error"))
        }
    }
}

#[tracing::instrument(skip(agent))]
pub async fn missing_blobs(agent: &BskyAgent) -> Result<Vec<RecordBlob>, PdsError> {
    use bsky_sdk::api::com::atproto::repo::list_missing_blobs::{Parameters, ParametersData};
    let result = agent
        .api
        .com
        .atproto
        .repo
        .list_missing_blobs(Parameters {
            data: ParametersData {
                cursor: None,
                limit: None,
            },
            extra_data: Ipld::Null,
        })
        .await;
    match result {
        Ok(output) => {
            tracing::info!("{:?}", output);
            Ok(output.blobs.clone())
        }
        Err(e) => {
            tracing::error!("{:?}", e);
            Err(PdsError::Validation)
        }
    }
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
            tracing::info!("Successfully signed token");
            Ok(())
        }
        Err(e) => {
            eprintln!("Error importing: {:?}", e);
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
            tokio::fs::write(did.as_str().to_string() + ".car", output)
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
