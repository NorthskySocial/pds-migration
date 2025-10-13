use crate::errors::ApiError;
use crate::post;
use actix_web::web::Json;
use actix_web::HttpResponse;
use pdsmigration_common::{MigrationError, RequestTokenRequest};

#[tracing::instrument(skip(req))]
#[post("/request-token")]
pub async fn request_token_api(req: Json<RequestTokenRequest>) -> Result<HttpResponse, ApiError> {
    tracing::info!("Request token request received");
    pdsmigration_common::request_token_api(req.into_inner())
        .await
        .map_err(|e| {
            tracing::error!("Failed to request token: {}", e);
            match e {
                MigrationError::Validation { .. } => ApiError::Runtime {
                    message: "Unexpected error occurred".to_string(),
                },
                MigrationError::Upstream { .. } => ApiError::Runtime {
                    message: "Unexpected error occurred".to_string(),
                },
                MigrationError::Runtime { .. } => ApiError::Runtime {
                    message: "Unexpected error occurred".to_string(),
                },
                MigrationError::RateLimitReached => ApiError::Runtime {
                    message: "Unexpected error occurred".to_string(),
                },
                MigrationError::Authentication { .. } => todo!(),
            }
        })?;
    Ok(HttpResponse::Ok().finish())
}
