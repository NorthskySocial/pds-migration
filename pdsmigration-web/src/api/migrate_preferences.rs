use crate::errors::ApiError;
use crate::post;
use actix_web::web::Json;
use actix_web::HttpResponse;
use pdsmigration_common::MigratePreferencesRequest;

#[tracing::instrument(skip(req))]
#[post("/migrate-preferences")]
pub async fn migrate_preferences_api(
    req: Json<MigratePreferencesRequest>,
) -> Result<HttpResponse, ApiError> {
    tracing::info!("Migrate preferences request received");
    pdsmigration_common::migrate_preferences_api(req.into_inner())
        .await
        .map_err(|e| {
            tracing::error!("Failed to migrate preferences: {}", e);
            ApiError::Runtime {
                message: "Unexpected error occurred".to_string(),
            }
        })?;
    Ok(HttpResponse::Ok().finish())
}
