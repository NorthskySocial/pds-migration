use crate::errors::{ApiError, ApiErrorBody};
use crate::post;
use actix_web::web::Json;
use actix_web::HttpResponse;
use pdsmigration_common::{MigrationError, RequestTokenRequest};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct RequestTokenApiRequest {
    pub pds_host: String,
    pub did: String,
    pub token: String,
}

impl From<RequestTokenApiRequest> for RequestTokenRequest {
    fn from(req: RequestTokenApiRequest) -> Self {
        Self {
            pds_host: req.pds_host,
            did: req.did,
            token: req.token,
        }
    }
}

#[utoipa::path(
    post,
    path = "/request-token",
    request_body = RequestTokenApiRequest,
    responses(
        (status = 200, description = "PLC Action Token requested successfully"),
        (status = 400, description = "Invalid request", body = ApiErrorBody, content_type = "application/json")
    ),
    tag = "pdsmigration-web"
)]
#[tracing::instrument(skip(req))]
#[post("/request-token")]
pub async fn request_token_api(
    req: Json<RequestTokenApiRequest>,
) -> Result<HttpResponse, ApiError> {
    tracing::info!("Request token request received");
    let req = req.into_inner();
    pdsmigration_common::request_token_api(req.into())
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
