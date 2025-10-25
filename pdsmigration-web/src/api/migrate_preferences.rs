use crate::errors::{ApiError, ApiErrorBody};
use crate::post;
use actix_web::web::Json;
use actix_web::HttpResponse;
use pdsmigration_common::MigratePreferencesRequest;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct MigratePreferencesApiRequest {
    pub destination: String,
    pub destination_token: String,
    pub origin: String,
    pub did: String,
    pub origin_token: String,
}

impl From<MigratePreferencesApiRequest> for MigratePreferencesRequest {
    fn from(req: MigratePreferencesApiRequest) -> Self {
        Self {
            destination: req.destination,
            destination_token: req.destination_token,
            origin: req.origin,
            did: req.did,
            origin_token: req.origin_token,
        }
    }
}

#[utoipa::path(
    post,
    path = "/migrate-preferences",
    request_body = MigratePreferencesApiRequest,
    responses(
        (status = 200, description = "User preferences migrated successfully"),
        (status = 400, description = "Invalid request", body = ApiErrorBody, content_type = "application/json")
    ),
    tag = "pdsmigration-web"
)]
#[tracing::instrument(skip(req))]
#[post("/migrate-preferences")]
pub async fn migrate_preferences_api(
    req: Json<MigratePreferencesApiRequest>,
) -> Result<HttpResponse, ApiError> {
    tracing::info!("Migrate preferences request received");
    let req = req.into_inner();
    pdsmigration_common::migrate_preferences_api(req.into()).await?;
    Ok(HttpResponse::Ok().finish())
}
