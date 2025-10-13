use crate::errors::ApiError;
use crate::{post, APPLICATION_JSON};
use actix_web::web::Json;
use actix_web::HttpResponse;
use pdsmigration_common::MissingBlobsRequest;

#[tracing::instrument(skip(req))]
#[post("/missing-blobs")]
pub async fn missing_blobs_api(req: Json<MissingBlobsRequest>) -> Result<HttpResponse, ApiError> {
    tracing::info!("Missing blobs request received");
    let response = pdsmigration_common::missing_blobs_api(req.into_inner()).await?;
    Ok(HttpResponse::Ok()
        .content_type(APPLICATION_JSON)
        .json(response))
}
