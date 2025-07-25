mod errors;

use crate::errors::ApiError;
use actix_web::dev::Server;
use actix_web::web::Json;
use actix_web::{middleware, post, App, HttpResponse, HttpServer};
use dotenvy::dotenv;
use pdsmigration_common::{
    ActivateAccountRequest, CreateAccountApiRequest, DeactivateAccountRequest, ExportBlobsRequest,
    ExportPDSRequest, ImportPDSRequest, MigratePlcRequest, MigratePreferencesRequest,
    MissingBlobsRequest, RequestTokenRequest, ServiceAuthRequest, UploadBlobsRequest,
};
use std::{env, io};

pub const APPLICATION_JSON: &str = "application/json";

fn init_http_server(server_port: &str, worker_count: &str) -> Server {
    HttpServer::new(move || {
        App::new()
            .wrap(middleware::Logger::default())
            .service(request_token_api)
            .service(create_account_api)
            .service(export_pds_api)
            .service(import_pds_api)
            .service(missing_blobs_api)
            .service(export_blobs_api)
            .service(upload_blobs_api)
            .service(activate_account_api)
            .service(deactivate_account_api)
            .service(migrate_preferences_api)
            .service(migrate_plc_api)
            .service(get_service_auth_api)
    })
    .bind(format!("0.0.0.0:{server_port}"))
    .unwrap()
    .workers(worker_count.parse::<usize>().unwrap_or(2))
    .run()
}

#[actix_rt::main]
async fn main() -> io::Result<()> {
    dotenv().ok();
    env::set_var("RUST_LOG", "actix_web=debug,actix_server=debug");
    env_logger::init();

    let subscriber = tracing_subscriber::FmtSubscriber::new();
    tracing::subscriber::set_global_default(subscriber).unwrap();

    // Get Environment Variables
    let server_port = env::var("SERVER_PORT").unwrap_or("9090".to_string());
    let worker_count = env::var("WORKER_COUNT").unwrap_or("2".to_string());

    // Start Http Server
    let server = init_http_server(server_port.as_str(), worker_count.as_str());
    server.await
}

#[post("/service-auth")]
pub async fn get_service_auth_api(req: Json<ServiceAuthRequest>) -> Result<HttpResponse, ApiError> {
    let response = pdsmigration_common::get_service_auth_api(req.into_inner()).await?;
    Ok(HttpResponse::Ok().json(response))
}

#[tracing::instrument(skip(req))]
#[post("/create-account")]
pub async fn create_account_api(
    req: Json<CreateAccountApiRequest>,
) -> Result<HttpResponse, ApiError> {
    pdsmigration_common::create_account_api(req.into_inner()).await?;
    Ok(HttpResponse::Ok().finish())
}

#[tracing::instrument]
#[post("/export-repo")]
pub async fn export_pds_api(req: Json<ExportPDSRequest>) -> Result<HttpResponse, ApiError> {
    pdsmigration_common::export_pds_api(req.into_inner()).await?;
    Ok(HttpResponse::Ok().finish())
}

#[tracing::instrument]
#[post("/import-repo")]
pub async fn import_pds_api(req: Json<ImportPDSRequest>) -> Result<HttpResponse, ApiError> {
    let endpoint_url = env::var("ENDPOINT").unwrap();
    let config = aws_config::from_env()
        .region("auto")
        .endpoint_url(endpoint_url)
        .load()
        .await;
    let client = aws_sdk_s3::Client::new(&config);
    let bucket_name = "migration".to_string();
    let file_name = req.did.as_str().to_string() + ".car";
    let key = "migration".to_string() + req.did.as_str() + ".car";
    let body =
        aws_sdk_s3::primitives::ByteStream::from_path(std::path::Path::new(file_name.as_str()))
            .await;
    match client
        .put_object()
        .bucket(&bucket_name)
        .key(&key)
        .body(body.unwrap())
        .send()
        .await
    {
        Ok(output) => {
            tracing::info!("{:?}", output);
        }
        Err(e) => {
            tracing::error!("{:?}", e);
            return Err(ApiError::Validation);
        }
    }
    pdsmigration_common::import_pds_api(req.into_inner()).await?;
    Ok(HttpResponse::Ok().content_type(APPLICATION_JSON).finish())
}

#[tracing::instrument]
#[post("/missing-blobs")]
pub async fn missing_blobs_api(req: Json<MissingBlobsRequest>) -> Result<HttpResponse, ApiError> {
    let response = pdsmigration_common::missing_blobs_api(req.into_inner()).await?;
    Ok(HttpResponse::Ok()
        .content_type(APPLICATION_JSON)
        .body(response))
}

#[tracing::instrument]
#[post("/export-blobs")]
pub async fn export_blobs_api(req: Json<ExportBlobsRequest>) -> Result<HttpResponse, ApiError> {
    pdsmigration_common::export_blobs_api(req.into_inner()).await?;
    Ok(HttpResponse::Ok().content_type(APPLICATION_JSON).finish())
}

#[tracing::instrument]
#[post("/upload-blobs")]
pub async fn upload_blobs_api(req: Json<UploadBlobsRequest>) -> Result<HttpResponse, ApiError> {
    pdsmigration_common::upload_blobs_api(req.into_inner()).await?;
    Ok(HttpResponse::Ok().content_type(APPLICATION_JSON).finish())
}

#[tracing::instrument]
#[post("/activate-account")]
pub async fn activate_account_api(
    req: Json<ActivateAccountRequest>,
) -> Result<HttpResponse, ApiError> {
    pdsmigration_common::activate_account_api(req.into_inner()).await?;
    Ok(HttpResponse::Ok().content_type(APPLICATION_JSON).finish())
}

#[tracing::instrument]
#[post("/deactivate-account")]
pub async fn deactivate_account_api(
    req: Json<DeactivateAccountRequest>,
) -> Result<HttpResponse, ApiError> {
    pdsmigration_common::deactivate_account_api(req.into_inner()).await?;
    Ok(HttpResponse::Ok().content_type(APPLICATION_JSON).finish())
}

#[tracing::instrument]
#[post("/migrate-preferences")]
pub async fn migrate_preferences_api(
    req: Json<MigratePreferencesRequest>,
) -> Result<HttpResponse, ApiError> {
    pdsmigration_common::migrate_preferences_api(req.into_inner()).await?;
    Ok(HttpResponse::Ok().content_type(APPLICATION_JSON).finish())
}

#[tracing::instrument]
#[post("/request-token")]
pub async fn request_token_api(req: Json<RequestTokenRequest>) -> Result<HttpResponse, ApiError> {
    pdsmigration_common::request_token_api(req.into_inner()).await?;
    Ok(HttpResponse::Ok().content_type(APPLICATION_JSON).finish())
}

#[tracing::instrument(skip(req))]
#[post("/migrate-plc")]
pub async fn migrate_plc_api(req: Json<MigratePlcRequest>) -> Result<HttpResponse, ApiError> {
    pdsmigration_common::migrate_plc_api(req.into_inner()).await?;
    Ok(HttpResponse::Ok().content_type(APPLICATION_JSON).finish())
}
