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
    })
    .bind(format!("0.0.0.0:{server_port}"))?
    .workers(worker_count.parse::<usize>().unwrap_or(2))
    .run();

    Ok(server)
}

#[actix_rt::main]
async fn main() -> io::Result<()> {
    dotenv().ok();
    env::set_var("RUST_LOG", "actix_web=debug,actix_server=debug");
    env_logger::init();

    let subscriber = tracing_subscriber::FmtSubscriber::new();
    tracing::subscriber::set_global_default(subscriber).map_err(|e| {
        io::Error::new(
            io::ErrorKind::Other,
            format!("Failed to set tracing subscriber: {}", e),
        )
    })?;

    // Get Environment Variables
    let server_port = env::var("SERVER_PORT").unwrap_or("9090".to_string());
    let worker_count = env::var("WORKER_COUNT").unwrap_or("2".to_string());

    // Start Http Server
    let server = init_http_server(server_port.as_str(), worker_count.as_str())?;
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
    let endpoint_url = env::var("ENDPOINT").map_err(|_| ApiError::Runtime)?;
    let config = aws_config::from_env()
        .region("auto")
        .endpoint_url(endpoint_url)
        .load()
        .await;
    let client = aws_sdk_s3::Client::new(&config);
    let bucket_name = "migration".to_string();
    let file_name = req.did.as_str().to_string() + ".car";
    let key = "migration".to_string() + req.did.as_str() + ".car";
    let body = match aws_sdk_s3::primitives::ByteStream::from_path(std::path::Path::new(
        file_name.as_str(),
    ))
    .await
    {
        Ok(body) => body,
        Err(e) => {
            tracing::error!("Failed to create ByteStream from file: {:?}", e);
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

    // Note: Additional utility function tests can be added here as needed
}
