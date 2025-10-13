use crate::agent::login_helper;
use crate::{build_agent, MigrationError};

#[tracing::instrument]
pub async fn activate_account(
    pds_host: &str,
    did: &str,
    token: &str,
) -> Result<(), MigrationError> {
    let agent = build_agent().await?;
    login_helper(&agent, pds_host, did, token).await?;
    agent
        .api
        .com
        .atproto
        .server
        .activate_account()
        .await
        .map_err(|error| MigrationError::Upstream {
            message: error.to_string(),
        })?;
    Ok(())
}
