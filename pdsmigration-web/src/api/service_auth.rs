use crate::errors::{ApiError, ApiErrorBody};
use crate::post;
use actix_web::web::Json;
use actix_web::HttpResponse;
use pdsmigration_common::ServiceAuthRequest;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct ServiceAuthApiRequest {
    pub pds_host: String,
    pub aud: String,
    pub did: String,
    pub token: String,
}

impl From<ServiceAuthApiRequest> for ServiceAuthRequest {
    fn from(req: ServiceAuthApiRequest) -> Self {
        Self {
            pds_host: req.pds_host,
            aud: req.aud,
            did: req.did,
            token: req.token,
        }
    }
}

#[derive(Serialize, Debug, Deserialize, ToSchema)]
struct ServiceAuthResponse {
    token: String,
}

#[utoipa::path(
    post,
    path = "/service-auth",
    request_body = ServiceAuthApiRequest,
    responses(
        (status = 200, description = "Service Auth token successfully requested", body = ServiceAuthResponse, content_type = "application/json"),
        (status = 400, description = "Invalid request", body = ApiErrorBody, content_type = "application/json")
    ),
    tag = "pdsmigration-web"
)]
#[tracing::instrument(skip(req), fields(did = %req.did, pds_host = %req.pds_host, aud = %req.aud))]
#[post("/service-auth")]
pub async fn get_service_auth_api(
    req: Json<ServiceAuthApiRequest>,
) -> Result<HttpResponse, ApiError> {
    tracing::info!("Service auth request received");
    let req = req.into_inner();
    let response = pdsmigration_common::get_service_auth_api(req.into()).await?;
    let response = ServiceAuthResponse { token: response };
    Ok(HttpResponse::Ok().json(response))
}
