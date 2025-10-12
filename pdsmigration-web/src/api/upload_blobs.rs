use crate::errors::ApiError;
use crate::post;
use actix_web::web::Json;
use actix_web::HttpResponse;
use pdsmigration_common::{MigrationError, UploadBlobsRequest};

#[tracing::instrument(skip(req))]
#[post("/upload-blobs")]
pub async fn upload_blobs_api(req: Json<UploadBlobsRequest>) -> Result<HttpResponse, ApiError> {
    pdsmigration_common::upload_blobs_api(req.into_inner())
        .await
        .map_err(|e| {
            tracing::error!("Failed to upload blobs: {}", e);
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
            }
        })?;
    Ok(HttpResponse::Ok().finish())
}
