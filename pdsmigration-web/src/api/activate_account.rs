use crate::errors::ApiError;
use crate::post;
use actix_web::web::Json;
use actix_web::HttpResponse;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct ActivateAccountRequest {
    pub pds_host: String,
    pub did: String,
    pub token: String,
}

#[tracing::instrument(skip(req), fields(did = %req.did, pds_host = %req.pds_host))]
#[post("/activate-account")]
pub async fn activate_account_api(
    req: Json<ActivateAccountRequest>,
) -> Result<HttpResponse, ApiError> {
    let req = req.into_inner();
    let did = req.did.clone();
    let token = req.token.clone();
    let pds_host = req.pds_host.clone();
    pdsmigration_common::activate_account(pds_host.as_str(), did.as_str(), token.as_str())
        .await
        .map_err(|e| {
            tracing::error!("Failed to activate account: {}", e);
            ApiError::Runtime {
                message: e.to_string(),
            }
        })?;
    Ok(HttpResponse::Ok().finish())
}
