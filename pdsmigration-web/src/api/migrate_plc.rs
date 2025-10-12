use crate::errors::ApiError;
use crate::post;
use actix_web::web::Json;
use actix_web::HttpResponse;
use pdsmigration_common::MigratePlcRequest;

#[tracing::instrument(skip(req))]
#[post("/migrate-plc")]
pub async fn migrate_plc_api(req: Json<MigratePlcRequest>) -> Result<HttpResponse, ApiError> {
    pdsmigration_common::migrate_plc_api(req.into_inner()).await?;
    Ok(HttpResponse::Ok().finish())
}
