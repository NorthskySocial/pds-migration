use crate::agent::{get_service_auth, login_helper};
use crate::{build_agent, MigrationError};
use serde::{Deserialize, Serialize};

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
pub async fn get_service_auth_api(req: ServiceAuthRequest) -> Result<String, MigrationError> {
    let agent = build_agent().await?;
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
