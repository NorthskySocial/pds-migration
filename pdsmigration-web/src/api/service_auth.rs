use crate::errors::ApiError;
use crate::post;
use actix_web::web::Json;
use actix_web::HttpResponse;
use pdsmigration_common::ServiceAuthRequest;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Debug, Deserialize)]
struct ServiceAuthResponse {
    token: String,
}

#[tracing::instrument(skip(req), fields(did = %req.did, pds_host = %req.pds_host, aud = %req.aud))]
#[post("/service-auth")]
pub async fn get_service_auth_api(req: Json<ServiceAuthRequest>) -> Result<HttpResponse, ApiError> {
    tracing::info!("Service auth request received");
    let response = pdsmigration_common::get_service_auth_api(req.into_inner()).await?;
    let response = ServiceAuthResponse { token: response };
    Ok(HttpResponse::Ok().json(response))
}
