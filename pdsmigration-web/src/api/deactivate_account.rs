use crate::errors::ApiError;
use crate::post;
use actix_web::web::Json;
use actix_web::HttpResponse;
use pdsmigration_common::DeactivateAccountRequest;

#[tracing::instrument(skip(req), fields(did = %req.did, pds_host = %req.pds_host))]
#[post("/deactivate-account")]
pub async fn deactivate_account_api(
    req: Json<DeactivateAccountRequest>,
) -> Result<HttpResponse, ApiError> {
    pdsmigration_common::deactivate_account_api(req.into_inner())
        .await
        .map_err(|e| {
            tracing::error!("Failed to deactivate account: {}", e);
            ApiError::Runtime {
                message: e.to_string(),
            }
        })?;
    Ok(HttpResponse::Ok().finish())
}
