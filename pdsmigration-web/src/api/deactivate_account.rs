use crate::errors::{ApiError, ApiErrorBody};
use crate::post;
use actix_web::web::Json;
use actix_web::HttpResponse;
use pdsmigration_common::DeactivateAccountRequest;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct DeactivateAccountApiRequest {
    pub pds_host: String,
    pub did: String,
    pub token: String,
}

impl From<DeactivateAccountApiRequest> for DeactivateAccountRequest {
    fn from(req: DeactivateAccountApiRequest) -> Self {
        Self {
            pds_host: req.pds_host,
            did: req.did,
            token: req.token,
        }
    }
}

#[utoipa::path(
    post,
    path = "/deactivate-account",
    request_body = DeactivateAccountApiRequest,
    responses(
        (status = 200, description = "Account deactivated successfully"),
        (status = 400, description = "Invalid request", body = ApiErrorBody, content_type = "application/json")
    ),
    tag = "pdsmigration-web"
)]
#[tracing::instrument(skip(req), fields(did = %req.did, pds_host = %req.pds_host))]
#[post("/deactivate-account")]
pub async fn deactivate_account_api(
    req: Json<DeactivateAccountApiRequest>,
) -> Result<HttpResponse, ApiError> {
    tracing::info!("Deactivate account request received");
    let req = req.into_inner();
    pdsmigration_common::deactivate_account_api(req.into()).await?;
    Ok(HttpResponse::Ok().finish())
}
