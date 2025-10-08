mod errors;

use crate::errors::ApiError;
use actix_web::dev::Server;
use actix_web::web::Json;
use actix_web::{get, middleware, post, App, HttpResponse, HttpServer, Responder};
use dotenvy::dotenv;
use pdsmigration_common::{
    ActivateAccountRequest, CreateAccountApiRequest, DeactivateAccountRequest, ExportBlobsRequest,
    ExportPDSRequest, ImportPDSRequest, MigratePlcRequest, MigratePreferencesRequest,
    MissingBlobsRequest, RequestTokenRequest, ServiceAuthRequest, UploadBlobsRequest,
};
use std::{env, io};

pub const APPLICATION_JSON: &str = "application/json";

fn init_http_server(server_port: &str, worker_count: &str) -> io::Result<Server> {
    let server = HttpServer::new(move || {
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
            .service(health_check)
    })
    .bind(format!("0.0.0.0:{server_port}"))?
    .workers(worker_count.parse::<usize>().unwrap_or(2))
    .run();

    Ok(server)
}

#[actix_rt::main]
async fn main() -> io::Result<()> {
    dotenv().ok();

    // Initialize tracing subscriber with better formatting
    let subscriber = tracing_subscriber::fmt()
        .with_target(true)
        .with_thread_ids(true)
        .with_level(true)
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| {
                tracing_subscriber::EnvFilter::new("info,pdsmigration_web=debug")
            }),
        )
        .finish();

    tracing::subscriber::set_global_default(subscriber).map_err(|e| {
        io::Error::new(
            io::ErrorKind::Other,
            format!("Failed to set tracing subscriber: {}", e),
        )
    })?;

    // Get Environment Variables
    let server_port = env::var("SERVER_PORT").unwrap_or("9090".to_string());
    let worker_count = env::var("WORKER_COUNT").unwrap_or("2".to_string());

    tracing::info!("Starting PDS Migration Web API");
    tracing::info!("Server port: {}", server_port);
    tracing::info!("Worker count: {}", worker_count);

    // Start Http Server
    let server = init_http_server(server_port.as_str(), worker_count.as_str())?;
    tracing::info!("Server started successfully on 0.0.0.0:{}", server_port);

    server.await
}

#[get("/health")]
async fn health_check() -> impl Responder {
    HttpResponse::Ok().body("OK")
}

#[tracing::instrument(skip(req), fields(did = %req.did, pds_host = %req.pds_host))]
#[post("/service-auth")]
pub async fn get_service_auth_api(req: Json<ServiceAuthRequest>) -> Result<HttpResponse, ApiError> {
    tracing::info!("Requesting service auth");
    let response = pdsmigration_common::get_service_auth_api(req.into_inner()).await?;
    tracing::info!("Service auth successful");
    Ok(HttpResponse::Ok().json(response))
}

#[tracing::instrument(skip(req), fields(did = %req.did, handle = %req.handle, pds_host = %req.pds_host
))]
#[post("/create-account")]
pub async fn create_account_api(
    req: Json<CreateAccountApiRequest>,
) -> Result<HttpResponse, ApiError> {
    tracing::info!("Creating account");
    pdsmigration_common::create_account_api(req.into_inner()).await?;
    tracing::info!("Account created successfully");
    Ok(HttpResponse::Ok().finish())
}

#[tracing::instrument(skip(req), fields(did = %req.did, pds_host = %req.pds_host))]
#[post("/export-repo")]
pub async fn export_pds_api(req: Json<ExportPDSRequest>) -> Result<HttpResponse, ApiError> {
    tracing::info!("Exporting repository");
    pdsmigration_common::export_pds_api(req.into_inner()).await?;
    tracing::info!("Repository exported successfully");
    Ok(HttpResponse::Ok().finish())
}

#[tracing::instrument(skip(req), fields(did = %req.did, pds_host = %req.pds_host))]
#[post("/import-repo")]
pub async fn import_pds_api(req: Json<ImportPDSRequest>) -> Result<HttpResponse, ApiError> {
    tracing::info!("Starting repository import");

    let endpoint_url = env::var("ENDPOINT").map_err(|e| {
        tracing::error!("Failed to get ENDPOINT environment variable: {}", e);
        ApiError::Runtime
    })?;

    tracing::debug!("Loading AWS config with endpoint: {}", endpoint_url);
    let config = aws_config::from_env()
        .region("auto")
        .endpoint_url(&endpoint_url)
        .load()
        .await;
    let client = aws_sdk_s3::Client::new(&config);

    let bucket_name = "migration".to_string();
    let file_name = req.did.as_str().to_string() + ".car";
    let key = "migration".to_string() + req.did.as_str() + ".car";

    tracing::debug!(
        "Uploading file {} to S3 bucket {} with key {}",
        file_name,
        bucket_name,
        key
    );

    let body = match aws_sdk_s3::primitives::ByteStream::from_path(std::path::Path::new(
        file_name.as_str(),
    ))
    .await
    {
        Ok(body) => {
            tracing::debug!("Successfully created ByteStream from file");
            body
        }
        Err(e) => {
            tracing::error!(
                "Failed to create ByteStream from file {}: {:?}",
                file_name,
                e
            );
            return Err(ApiError::Runtime);
        }
    };

    match client
        .put_object()
        .bucket(&bucket_name)
        .key(&key)
        .body(body)
        .send()
        .await
    {
        Ok(output) => {
            tracing::info!(
                "Successfully uploaded to S3: bucket={}, key={}",
                bucket_name,
                key
            );
            tracing::debug!("S3 upload output: {:?}", output);
        }
        Err(e) => {
            tracing::error!(
                "Failed to upload to S3: bucket={}, key={}, error={:?}",
                bucket_name,
                key,
                e
            );
            return Err(ApiError::Validation);
        }
    }

    tracing::info!("Importing repository to PDS");
    pdsmigration_common::import_pds_api(req.into_inner()).await?;
    tracing::info!("Repository imported successfully");

    Ok(HttpResponse::Ok().content_type(APPLICATION_JSON).finish())
}

#[tracing::instrument(skip(req), fields(did = %req.did, pds_host = %req.pds_host))]
#[post("/missing-blobs")]
pub async fn missing_blobs_api(req: Json<MissingBlobsRequest>) -> Result<HttpResponse, ApiError> {
    tracing::info!("Checking for missing blobs");
    let response = pdsmigration_common::missing_blobs_api(req.into_inner()).await?;
    tracing::info!("Missing blobs check completed");
    Ok(HttpResponse::Ok()
        .content_type(APPLICATION_JSON)
        .body(response))
}

#[tracing::instrument(skip(req), fields(did = %req.did, origin = %req.origin, destination = %req.destination
))]
#[post("/export-blobs")]
pub async fn export_blobs_api(req: Json<ExportBlobsRequest>) -> Result<HttpResponse, ApiError> {
    tracing::info!("Exporting blobs");
    pdsmigration_common::export_blobs_api(req.into_inner()).await?;
    tracing::info!("Blobs exported successfully");
    Ok(HttpResponse::Ok().content_type(APPLICATION_JSON).finish())
}

#[tracing::instrument(skip(req), fields(did = %req.did, pds_host = %req.pds_host))]
#[post("/upload-blobs")]
pub async fn upload_blobs_api(req: Json<UploadBlobsRequest>) -> Result<HttpResponse, ApiError> {
    tracing::info!("Uploading blobs");
    pdsmigration_common::upload_blobs_api(req.into_inner()).await?;
    tracing::info!("Blobs uploaded successfully");
    Ok(HttpResponse::Ok().content_type(APPLICATION_JSON).finish())
}

#[tracing::instrument(skip(req), fields(did = %req.did, pds_host = %req.pds_host))]
#[post("/activate-account")]
pub async fn activate_account_api(
    req: Json<ActivateAccountRequest>,
) -> Result<HttpResponse, ApiError> {
    tracing::info!("Activating account");
    pdsmigration_common::activate_account_api(req.into_inner()).await?;
    tracing::info!("Account activated successfully");
    Ok(HttpResponse::Ok().content_type(APPLICATION_JSON).finish())
}

#[tracing::instrument(skip(req), fields(did = %req.did, pds_host = %req.pds_host))]
#[post("/deactivate-account")]
pub async fn deactivate_account_api(
    req: Json<DeactivateAccountRequest>,
) -> Result<HttpResponse, ApiError> {
    tracing::info!("Deactivating account");
    pdsmigration_common::deactivate_account_api(req.into_inner()).await?;
    tracing::info!("Account deactivated successfully");
    Ok(HttpResponse::Ok().content_type(APPLICATION_JSON).finish())
}

#[tracing::instrument(skip(req), fields(did = %req.did, origin = %req.origin, destination = %req.destination
))]
#[post("/migrate-preferences")]
pub async fn migrate_preferences_api(
    req: Json<MigratePreferencesRequest>,
) -> Result<HttpResponse, ApiError> {
    tracing::info!("Migrating preferences");
    pdsmigration_common::migrate_preferences_api(req.into_inner()).await?;
    tracing::info!("Preferences migrated successfully");
    Ok(HttpResponse::Ok().content_type(APPLICATION_JSON).finish())
}

#[tracing::instrument(skip(req), fields(did = %req.did, pds_host = %req.pds_host))]
#[post("/request-token")]
pub async fn request_token_api(req: Json<RequestTokenRequest>) -> Result<HttpResponse, ApiError> {
    tracing::info!("Requesting token");
    pdsmigration_common::request_token_api(req.into_inner()).await?;
    tracing::info!("Token requested successfully");
    Ok(HttpResponse::Ok().content_type(APPLICATION_JSON).finish())
}

#[tracing::instrument(skip(req), fields(did = %req.did, origin = %req.origin, destination = %req.destination
))]
#[post("/migrate-plc")]
pub async fn migrate_plc_api(req: Json<MigratePlcRequest>) -> Result<HttpResponse, ApiError> {
    tracing::info!("Migrating PLC");
    pdsmigration_common::migrate_plc_api(req.into_inner()).await?;
    tracing::info!("PLC migrated successfully");
    Ok(HttpResponse::Ok().content_type(APPLICATION_JSON).finish())
}

#[cfg(test)]
mod tests {
    use crate::{
        activate_account_api, create_account_api, deactivate_account_api, export_blobs_api,
        export_pds_api, get_service_auth_api, migrate_plc_api, migrate_preferences_api,
        missing_blobs_api, request_token_api, upload_blobs_api,
    };
    use actix_web::{test, App};
    use pdsmigration_common::{
        ActivateAccountRequest, CreateAccountApiRequest, DeactivateAccountRequest,
        ExportBlobsRequest, ExportPDSRequest, MigratePlcRequest, MigratePreferencesRequest,
        MissingBlobsRequest, RequestTokenRequest, ServiceAuthRequest, UploadBlobsRequest,
    };

    #[actix_rt::test]
    async fn test_get_service_auth_api() {
        let app = test::init_service(App::new().service(get_service_auth_api)).await;

        let req_payload = ServiceAuthRequest {
            pds_host: "https://test.example.com".to_string(),
            aud: "https://audience.com".to_string(),
            did: "did:plc:test123".to_string(),
            token: "test_token".to_string(),
        };

        let req = test::TestRequest::post()
            .uri("/service-auth")
            .set_json(&req_payload)
            .to_request();

        // Note: This test would fail in CI without proper network setup
        // but demonstrates the test structure for API endpoints
        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success() || resp.status().is_client_error());
    }

    #[actix_rt::test]
    async fn test_create_account_api() {
        let app = test::init_service(App::new().service(create_account_api)).await;

        let req_payload = CreateAccountApiRequest {
            email: "test@example.com".to_string(),
            handle: "test.bsky.social".to_string(),
            invite_code: "invite123".to_string(),
            password: "password123".to_string(),
            token: "token123".to_string(),
            pds_host: "https://test.pds.com".to_string(),
            did: "did:plc:test123".to_string(),
            recovery_key: None,
        };

        let req = test::TestRequest::post()
            .uri("/create-account")
            .set_json(&req_payload)
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success() || resp.status().is_client_error());
    }

    #[actix_rt::test]
    async fn test_export_pds_api() {
        let app = test::init_service(App::new().service(export_pds_api)).await;

        let req_payload = ExportPDSRequest {
            did: "did:plc:test123".to_string(),
            token: "export_token".to_string(),
            pds_host: "https://test.pds.com".to_string(),
        };

        let req = test::TestRequest::post()
            .uri("/export-repo")
            .set_json(&req_payload)
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success() || resp.status().is_client_error());
    }

    #[actix_rt::test]
    async fn test_missing_blobs_api() {
        let app = test::init_service(App::new().service(missing_blobs_api)).await;

        let req_payload = MissingBlobsRequest {
            did: "did:plc:test123".to_string(),
            token: "blob_token".to_string(),
            pds_host: "https://test.pds.com".to_string(),
        };

        let req = test::TestRequest::post()
            .uri("/missing-blobs")
            .set_json(&req_payload)
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success() || resp.status().is_client_error());
    }

    #[actix_rt::test]
    async fn test_export_blobs_api() {
        let app = test::init_service(App::new().service(export_blobs_api)).await;

        let req_payload = ExportBlobsRequest {
            destination: "https://destination.pds.com".to_string(),
            origin: "https://origin.pds.com".to_string(),
            did: "did:plc:test123".to_string(),
            origin_token: "origin_token".to_string(),
            destination_token: "destination_token".to_string(),
        };

        let req = test::TestRequest::post()
            .uri("/export-blobs")
            .set_json(&req_payload)
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success() || resp.status().is_client_error());
    }

    #[actix_rt::test]
    async fn test_upload_blobs_api() {
        let app = test::init_service(App::new().service(upload_blobs_api)).await;

        let req_payload = UploadBlobsRequest {
            did: "did:plc:test123".to_string(),
            token: "upload_token".to_string(),
            pds_host: "https://test.pds.com".to_string(),
        };

        let req = test::TestRequest::post()
            .uri("/upload-blobs")
            .set_json(&req_payload)
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success() || resp.status().is_client_error());
    }

    #[actix_rt::test]
    async fn test_activate_account_api() {
        let app = test::init_service(App::new().service(activate_account_api)).await;

        let req_payload = ActivateAccountRequest {
            did: "did:plc:test123".to_string(),
            token: "activate_token".to_string(),
            pds_host: "https://test.pds.com".to_string(),
        };

        let req = test::TestRequest::post()
            .uri("/activate-account")
            .set_json(&req_payload)
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success() || resp.status().is_client_error());
    }

    #[actix_rt::test]
    async fn test_deactivate_account_api() {
        let app = test::init_service(App::new().service(deactivate_account_api)).await;

        let req_payload = DeactivateAccountRequest {
            did: "did:plc:test123".to_string(),
            token: "deactivate_token".to_string(),
            pds_host: "https://test.pds.com".to_string(),
        };

        let req = test::TestRequest::post()
            .uri("/deactivate-account")
            .set_json(&req_payload)
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success() || resp.status().is_client_error());
    }

    #[actix_rt::test]
    async fn test_migrate_preferences_api() {
        let app = test::init_service(App::new().service(migrate_preferences_api)).await;

        let req_payload = MigratePreferencesRequest {
            destination: "https://destination.pds.com".to_string(),
            destination_token: "destination_token".to_string(),
            origin: "https://origin.pds.com".to_string(),
            did: "did:plc:test123".to_string(),
            origin_token: "origin_token".to_string(),
        };

        let req = test::TestRequest::post()
            .uri("/migrate-preferences")
            .set_json(&req_payload)
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success() || resp.status().is_client_error());
    }

    #[actix_rt::test]
    async fn test_request_token_api() {
        let app = test::init_service(App::new().service(request_token_api)).await;

        let req_payload = RequestTokenRequest {
            pds_host: "https://test.pds.com".to_string(),
            did: "did:plc:test123".to_string(),
            token: "test_token".to_string(),
        };

        let req = test::TestRequest::post()
            .uri("/request-token")
            .set_json(&req_payload)
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success() || resp.status().is_client_error());
    }

    #[actix_rt::test]
    async fn test_migrate_plc_api() {
        let app = test::init_service(App::new().service(migrate_plc_api)).await;

        let req_payload = MigratePlcRequest {
            destination: "https://new.pds.com".to_string(),
            destination_token: "destination_token".to_string(),
            origin: "https://old.pds.com".to_string(),
            did: "did:plc:test123".to_string(),
            origin_token: "origin_token".to_string(),
            plc_signing_token: "plc_signing_token".to_string(),
            user_recovery_key: None,
        };

        let req = test::TestRequest::post()
            .uri("/migrate-plc")
            .set_json(&req_payload)
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success() || resp.status().is_client_error());
    }
}
