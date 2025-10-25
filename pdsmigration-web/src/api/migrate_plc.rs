use crate::errors::{ApiError, ApiErrorBody};
use crate::post;
use actix_web::web::Json;
use actix_web::HttpResponse;
use pdsmigration_common::MigratePlcRequest;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct MigratePlcApiRequest {
    pub destination: String,
    pub destination_token: String,
    pub origin: String,
    pub did: String,
    pub origin_token: String,
    pub plc_signing_token: String,
    #[serde(skip_serializing_if = "core::option::Option::is_none")]
    pub user_recovery_key: Option<String>,
}

impl From<MigratePlcApiRequest> for MigratePlcRequest {
    fn from(req: MigratePlcApiRequest) -> Self {
        Self {
            destination: req.destination,
            destination_token: req.destination_token,
            origin: req.origin,
            did: req.did,
            origin_token: req.origin_token,
            plc_signing_token: req.plc_signing_token,
            user_recovery_key: req.user_recovery_key,
        }
    }
}

#[utoipa::path(
    post,
    path = "/migrate-plc",
    request_body = MigratePlcApiRequest,
    responses(
        (status = 200, description = "PLC migrated successfully"),
        (status = 400, description = "Invalid request", body = ApiErrorBody, content_type = "application/json")
    ),
    tag = "pdsmigration-web"
)]
#[tracing::instrument(skip(req))]
#[post("/migrate-plc")]
pub async fn migrate_plc_api(req: Json<MigratePlcApiRequest>) -> Result<HttpResponse, ApiError> {
    tracing::info!("Migrate PLC request received");
    let req = req.into_inner();
    pdsmigration_common::migrate_plc_api(req.into()).await?;
    Ok(HttpResponse::Ok().finish())
}
