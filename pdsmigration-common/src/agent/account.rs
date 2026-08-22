use crate::{
    try_parse_error_response, CreateAccountInput, CreateAccountInputData, CreateAccountRequest,
    DeactivatedAccountInput, DeactivatedAccountInputData, MigrationError, APPLICATION_JSON,
    CREATE_ACCOUNT_PATH,
};
use bsky_sdk::BskyAgent;
use ipld_core::ipld::Ipld;

#[tracing::instrument(skip(account_request))]
pub async fn create_account(
    pds_host: &str,
    account_request: &CreateAccountRequest,
) -> Result<(), MigrationError> {
    let did_str = account_request.did.as_str();
    tracing::info!(
        "[{}] Creating account on {} - handle: {}, has_email: {}, has_invite_code: {}, has_password: {}, has_recovery_key: {}, has_service_token: {}",
        did_str,
        pds_host,
        account_request.handle.as_str(),
        account_request.email.is_some(),
        account_request.invite_code.is_some(),
        account_request.password.is_some(),
        account_request.recovery_key.is_some(),
        account_request.token.is_some()
    );
    let client = reqwest::Client::new();
    let request_body = serde_json::to_string(&CreateAccountInput {
        data: CreateAccountInputData {
            did: Some(account_request.did.clone()),
            email: account_request.email.clone(),
            handle: account_request.handle.clone(),
            invite_code: account_request.invite_code.clone(),
            password: account_request.password.clone(),
            plc_op: None,
            recovery_key: account_request.recovery_key.clone(),
            verification_code: account_request.verification_code.clone(),
            verification_phone: account_request.verification_phone.clone(),
        },
        extra_data: Ipld::Null,
    })
    .map_err(|error| {
        tracing::error!(
            "[{}] Failed to create account - Error mapping input data to JSON: {:?}",
            did_str,
            error
        );
        MigrationError::Runtime {
            message: "Failed to create account".to_string(),
        }
    })?;
    let mut request_builder = client
        .post(pds_host.to_string() + CREATE_ACCOUNT_PATH)
        .body(request_body)
        .header("Content-Type", APPLICATION_JSON);

    if let Some(token) = &account_request.token {
        request_builder = request_builder.bearer_auth(token);
    }

    let result = request_builder.send().await;
    match result {
        Ok(output) => match output.status() {
            reqwest::StatusCode::OK => {
                tracing::info!("[{}] Successfully created account", did_str);
            }
            reqwest::StatusCode::BAD_REQUEST => {
                let error_message = try_parse_error_response(output).await;

                tracing::error!(
                    "[{}] Failed to create account on {} - Bad Request: {}",
                    did_str,
                    pds_host,
                    error_message
                );
                return Err(MigrationError::Upstream {
                    message: error_message,
                });
            }
            _ => {
                let status = output.status();
                let response_text = output
                    .text()
                    .await
                    .unwrap_or_else(|_| "Unable to read response".to_string());
                tracing::error!(
                    "[{}] Failed to create account on {} - Received non-OK status on Create Account: {} - Response: {}",
                    did_str,
                    pds_host,
                    status,
                    response_text
                );
                return Err(MigrationError::Runtime {
                    message: "Failed to create account".to_string(),
                });
            }
        },
        Err(e) => {
            tracing::error!(
                "[{}] Failed to create account on {} - Request failed (timeout: {}, connect: {}, request: {}): {:?}",
                did_str,
                pds_host,
                e.is_timeout(),
                e.is_connect(),
                e.is_request(),
                e
            );
            return Err(MigrationError::Runtime {
                message: e.to_string(),
            });
        }
    }
    Ok(())
}

#[tracing::instrument(skip(agent))]
pub async fn deactivate_account(agent: &BskyAgent) -> Result<(), MigrationError> {
    let did = agent.did().await.clone();
    let did_str = did.as_ref().map(|d| d.as_str()).unwrap_or("unknown");
    agent
        .api
        .com
        .atproto
        .server
        .deactivate_account(DeactivatedAccountInput {
            data: DeactivatedAccountInputData { delete_after: None },
            extra_data: Ipld::Null,
        })
        .await
        .map_err(|error| {
            tracing::error!("[{}] Failed to deactivate account: {:?}", did_str, error);
            MigrationError::Runtime {
                message: error.to_string(),
            }
        })?;
    Ok(())
}

#[tracing::instrument(skip(agent))]
pub async fn activate_account_agent(agent: &BskyAgent) -> Result<(), MigrationError> {
    let did = agent.did().await.clone();
    let did_str = did.as_ref().map(|d| d.as_str()).unwrap_or("unknown");
    agent
        .api
        .com
        .atproto
        .server
        .activate_account()
        .await
        .map_err(|error| {
            tracing::error!("[{}] Failed to activate account: {:?}", did_str, error);
            MigrationError::Upstream {
                message: error.to_string(),
            }
        })?;
    Ok(())
}
