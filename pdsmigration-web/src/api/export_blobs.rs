use crate::errors::ApiError;
use crate::post;
use crate::Json;
use actix_web::HttpResponse;
use pdsmigration_common::ExportBlobsRequest;

#[tracing::instrument(skip(req))]
#[post("/export-blobs")]
pub async fn export_blobs_api(req: Json<ExportBlobsRequest>) -> Result<HttpResponse, ApiError> {
    tracing::info!("Exporting blobs");
    let result = pdsmigration_common::export_blobs_api(req.into_inner())
        .await
        .map_err(|error| ApiError::Runtime {
            message: error.to_string(),
        })?;
    tracing::info!("Blobs exported successfully");
    Ok(HttpResponse::Ok().json(result))
}
