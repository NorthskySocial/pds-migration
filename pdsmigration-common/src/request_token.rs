use crate::agent::{login_helper, request_token};
use crate::{build_agent, MigrationError};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct RequestTokenRequest {
    pub pds_host: String,
    pub did: String,
    pub token: String,
}

#[tracing::instrument(skip(req))]
pub async fn request_token_api(req: RequestTokenRequest) -> Result<(), MigrationError> {
    let agent = build_agent().await?;
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
