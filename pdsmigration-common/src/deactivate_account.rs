use crate::agent::{deactivate_account, login_helper};
use crate::{build_agent, MigrationError};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct DeactivateAccountRequest {
    pub pds_host: String,
    pub did: String,
    pub token: String,
}

#[tracing::instrument(skip(req))]
pub async fn deactivate_account_api(req: DeactivateAccountRequest) -> Result<(), MigrationError> {
    let agent = build_agent().await?;
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
