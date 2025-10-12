use crate::errors::ApiError;
use crate::post;
use actix_web::web::Json;
use actix_web::HttpResponse;
use pdsmigration_common::{MigrationError, ServiceAuthRequest};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Debug, Deserialize)]
struct ServiceAuthResponse {
    token: String,
}

#[tracing::instrument(skip(req), fields(did = %req.did, pds_host = %req.pds_host))]
#[post("/service-auth")]
pub async fn get_service_auth_api(req: Json<ServiceAuthRequest>) -> Result<HttpResponse, ApiError> {
    let response = pdsmigration_common::get_service_auth_api(req.into_inner())
        .await
        .map_err(|e| match e {
            MigrationError::Validation { .. } => ApiError::Runtime {
                message: "".to_string(),
            },
            MigrationError::Upstream { message } => {
                tracing::error!("Unexpected error occurred: {}", message);
                ApiError::Runtime {
                    message: "Unexpected error occurred".to_string(),
                }
            }
            MigrationError::Runtime { message } => {
                tracing::error!("Unexpected error occurred: {}", message);
                ApiError::Runtime {
                    message: "Unexpected error occurred".to_string(),
                }
            }
            MigrationError::RateLimitReached => {
                tracing::error!("This should never be reached");
                ApiError::Runtime {
                    message: "Unexpected error occurred".to_string(),
                }
            }
        })?;
    let response = ServiceAuthResponse { token: response };
    Ok(HttpResponse::Ok().json(response))
}
