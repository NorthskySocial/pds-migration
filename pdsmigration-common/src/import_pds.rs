use crate::agent::{account_import, login_helper};
use crate::{build_agent, MigrationError};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct ImportPDSRequest {
    pub pds_host: String,
    pub did: String,
    pub token: String,
}

#[tracing::instrument(skip(req))]
pub async fn import_pds_api(req: ImportPDSRequest) -> Result<(), MigrationError> {
    let agent = build_agent().await?;
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
