use crate::errors::{ApiError, ApiErrorBody};
use crate::{post, APPLICATION_JSON};
use actix_web::web::Json;
use actix_web::HttpResponse;
use pdsmigration_common::{MissingBlobsRequest, MissingBlobsResponse};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct MissingBlobsApiRequest {
    pub pds_host: String,
    pub did: String,
    pub token: String,
}

impl From<MissingBlobsApiRequest> for MissingBlobsRequest {
    fn from(req: MissingBlobsApiRequest) -> Self {
        Self {
            pds_host: req.pds_host,
            did: req.did,
            token: req.token,
        }
    }
}

#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct MissingBlobsApiResponse {
    pub missing_blobs: Vec<String>,
}

impl From<MissingBlobsResponse> for MissingBlobsApiResponse {
    fn from(req: MissingBlobsResponse) -> Self {
        Self {
            missing_blobs: req.missing_blobs,
        }
    }
}

#[utoipa::path(
    post,
    path = "/missing-blobs",
    request_body = MissingBlobsApiRequest,
    responses(
        (status = 200, description = "Missing blobs determined", body = MissingBlobsApiResponse, content_type = "application/json"),
        (status = 400, description = "Invalid request", body = ApiErrorBody, content_type = "application/json")
    ),
    tag = "pdsmigration-web"
)]
#[tracing::instrument(skip(req))]
#[post("/missing-blobs")]
pub async fn missing_blobs_api(
    req: Json<MissingBlobsApiRequest>,
) -> Result<HttpResponse, ApiError> {
    tracing::info!("Missing blobs request received");
    let req = req.into_inner();
    let response = pdsmigration_common::missing_blobs_api(req.into()).await?;
    let response: MissingBlobsApiResponse = response.into();
    Ok(HttpResponse::Ok()
        .content_type(APPLICATION_JSON)
        .json(response))
}
