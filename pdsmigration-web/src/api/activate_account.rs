use crate::errors::{ApiError, ApiErrorBody};
use crate::post;
use actix_web::web::Json;
use actix_web::HttpResponse;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct ActivateAccountApiRequest {
    pub pds_host: String,
    pub did: String,
    pub token: String,
}

#[tracing::instrument(skip(req), fields(did = %req.did, pds_host = %req.pds_host))]
#[utoipa::path(
    post,
    path = "/activate-account",
    request_body = ActivateAccountApiRequest,
    responses(
        (status = 200, description = "Account activated successfully"),
        (status = 400, description = "Invalid request", body = ApiErrorBody, content_type = "application/json")
    ),
    tag = "pdsmigration-web"
)]
#[post("/activate-account")]
pub async fn activate_account_api(
    req: Json<ActivateAccountApiRequest>,
) -> Result<HttpResponse, ApiError> {
    tracing::info!("Activate account request received");
    let req = req.into_inner();
    let did = req.did.clone();
    let token = req.token.clone();
    let pds_host = req.pds_host.clone();
    pdsmigration_common::activate_account(pds_host.as_str(), did.as_str(), token.as_str()).await?;
    Ok(HttpResponse::Ok().finish())
}
