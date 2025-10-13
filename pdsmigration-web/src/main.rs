mod api;
mod config;
mod errors;

use crate::api::{
    activate_account_api, create_account_api, deactivate_account_api, export_blobs_api,
    export_pds_api, get_service_auth_api, health_check, import_pds_api, migrate_plc_api,
    migrate_preferences_api, missing_blobs_api, request_token_api, upload_blobs_api,
};
use crate::config::AppConfig;
use actix_web::dev::Server;
use actix_web::web::Json;
use actix_web::{post, web, App, HttpResponse, HttpServer};
use actix_web_prom::PrometheusMetricsBuilder;
use dotenvy::dotenv;
use std::io;
use tracing_actix_web::TracingLogger;

pub const APPLICATION_JSON: &str = "application/json";

fn init_http_server(app_config: AppConfig) -> io::Result<Server> {
    let server_port = app_config.server.port;
    let worker_count = app_config.server.workers;
    let prometheus = PrometheusMetricsBuilder::new("api")
        .endpoint("/metrics")
        .build()
        .expect("Failed to build prometheus metrics");
    let server = HttpServer::new(move || {
        App::new()
            .wrap(prometheus.clone())
            .wrap(TracingLogger::default())
            .app_data(web::Data::new(app_config.clone()))
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
    .workers(worker_count)
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

    tracing::subscriber::set_global_default(subscriber)
        .map_err(|e| io::Error::other(format!("Failed to set tracing subscriber: {}", e)))?;

    // Load App Config
    let app_config = AppConfig::from_env();

    // Start Http Server
    let server = init_http_server(app_config.clone())?;
    tracing::info!(
        "Server started successfully on 0.0.0.0:{}",
        app_config.server.port
    );

    server.await
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{AppConfig, ExternalServices, ServerConfig};

    #[test]
    fn test_init_http_server_success() {
        let app_config = AppConfig {
            server: ServerConfig {
                port: 8080,
                workers: 2,
            },
            external_services: ExternalServices {
                s3_endpoint: "http://test-s3.example.com".to_string(),
            },
        };

        let result = init_http_server(app_config);
        assert!(result.is_ok(), "Expected successful server initialization");
    }

    // Integration test for server routes (without actually starting the server)
    #[actix_rt::test]
    async fn test_server_routes_configuration() {
        let app_config = AppConfig {
            server: ServerConfig {
                port: 8080,
                workers: 1,
            },
            external_services: ExternalServices {
                s3_endpoint: "http://test-s3.example.com".to_string(),
            },
        };

        // Test that we can create an app with all routes
        let app = actix_web::test::init_service(
            App::new()
                .app_data(web::Data::new(app_config.clone()))
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
                .service(health_check),
        )
        .await;

        // Test health endpoint
        let req = actix_web::test::TestRequest::get()
            .uri("/health")
            .to_request();

        let resp = actix_web::test::call_service(&app, req).await;
        assert_eq!(resp.status(), actix_web::http::StatusCode::OK);
    }
}
