use actix_web::{http::StatusCode, test, web, App};
use pdsmigration_web::{
    api::{
        activate_account_api, create_account_api, deactivate_account_api, export_blobs_api,
        export_pds_api, get_service_auth_api, health_check, import_pds_api, migrate_plc_api,
        migrate_preferences_api, missing_blobs_api, request_token_api, upload_blobs_api,
    },
    config::{AppConfig, ExternalServices, ServerConfig},
};
use serde_json::json;

#[cfg(test)]
mod integration_tests {
    use super::*;

    fn create_test_config() -> AppConfig {
        AppConfig {
            server: ServerConfig {
                port: 8080,
                workers: 1,
            },
            external_services: ExternalServices {
                s3_endpoint: "http://test-s3.example.com".to_string(),
            },
        }
    }

    #[actix_rt::test]
    async fn test_health_endpoint() {
        let app_config = create_test_config();

        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(app_config))
                .service(health_check),
        )
        .await;

        let req = test::TestRequest::get().uri("/health").to_request();

        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::OK);

        let body = test::read_body(resp).await;
        assert_eq!(body, "OK");
    }

    #[actix_rt::test]
    async fn test_all_routes_configured() {
        let app_config = create_test_config();

        // Test that we can create an app with all routes without errors
        let _app = test::init_service(
            App::new()
                .app_data(web::Data::new(app_config))
                .service(health_check)
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
                .service(get_service_auth_api),
        )
        .await;
    }

    #[actix_rt::test]
    async fn test_create_account_missing_fields() {
        let app_config = create_test_config();

        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(app_config))
                .service(create_account_api),
        )
        .await;

        let req = test::TestRequest::post()
            .uri("/create-account")
            .set_json(json!({}))
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
    }

    #[actix_rt::test]
    async fn test_create_account() {
        let app_config = create_test_config();

        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(app_config))
                .service(create_account_api),
        )
        .await;

        let req = test::TestRequest::post()
            .uri("/create-account")
            .set_json(json!({
                "email": "dummy@email.com",
                "handle": "dummy.handle",
                "invite_code": "dummy-invite-code",
                "password": "password",
                "token": "dummy-token",
                "pds_host": "https://dummy.pds.host",
                "did": "did:key"
            }))
            .to_request();

        let resp = test::call_service(&app, req).await;

        let status = resp.status().clone();

        let body = test::read_body(resp).await;
        eprintln!("{:?}", body);
        assert_eq!(status, StatusCode::OK);
    }

    #[actix_rt::test]
    async fn test_request_token_missing_fields() {
        let app_config = create_test_config();

        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(app_config))
                .service(request_token_api),
        )
        .await;

        let req = test::TestRequest::post()
            .uri("/request-token")
            .set_json(&json!({}))
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
    }

    #[actix_rt::test]
    async fn test_export_pds_missing_fields() {
        let app_config = create_test_config();

        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(app_config))
                .service(export_pds_api),
        )
        .await;

        let req = test::TestRequest::post()
            .uri("/export-pds")
            .set_json(&json!({}))
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::NOT_FOUND);
    }

    #[actix_rt::test]
    async fn test_import_pds_missing_fields() {
        let app_config = create_test_config();

        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(app_config))
                .service(import_pds_api),
        )
        .await;

        let req = test::TestRequest::post()
            .uri("/import-pds")
            .set_json(&json!({}))
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::NOT_FOUND);
    }

    #[actix_rt::test]
    async fn test_missing_blobs_missing_fields() {
        let app_config = create_test_config();

        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(app_config))
                .service(missing_blobs_api),
        )
        .await;

        let req = test::TestRequest::post()
            .uri("/missing-blobs")
            .set_json(&json!({}))
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
    }

    #[actix_rt::test]
    async fn test_export_blobs_missing_fields() {
        let app_config = create_test_config();

        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(app_config))
                .service(export_blobs_api),
        )
        .await;

        let req = test::TestRequest::post()
            .uri("/export-blobs")
            .set_json(&json!({}))
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
    }

    #[actix_rt::test]
    async fn test_upload_blobs_missing_fields() {
        let app_config = create_test_config();

        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(app_config))
                .service(upload_blobs_api),
        )
        .await;

        let req = test::TestRequest::post()
            .uri("/upload-blobs")
            .set_json(&json!({}))
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
    }

    #[actix_rt::test]
    async fn test_activate_account_missing_fields() {
        let app_config = create_test_config();

        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(app_config))
                .service(activate_account_api),
        )
        .await;

        let req = test::TestRequest::post()
            .uri("/activate-account")
            .set_json(&json!({}))
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
    }

    #[actix_rt::test]
    async fn test_deactivate_account_missing_fields() {
        let app_config = create_test_config();

        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(app_config))
                .service(deactivate_account_api),
        )
        .await;

        let req = test::TestRequest::post()
            .uri("/deactivate-account")
            .set_json(&json!({}))
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
    }

    #[actix_rt::test]
    async fn test_migrate_preferences_missing_fields() {
        let app_config = create_test_config();

        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(app_config))
                .service(migrate_preferences_api),
        )
        .await;

        let req = test::TestRequest::post()
            .uri("/migrate-preferences")
            .set_json(&json!({}))
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
    }

    #[actix_rt::test]
    async fn test_migrate_plc_missing_fields() {
        let app_config = create_test_config();

        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(app_config))
                .service(migrate_plc_api),
        )
        .await;

        let req = test::TestRequest::post()
            .uri("/migrate-plc")
            .set_json(&json!({}))
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
    }

    #[actix_rt::test]
    async fn test_get_service_auth_missing_fields() {
        let app_config = create_test_config();

        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(app_config))
                .service(get_service_auth_api),
        )
        .await;

        let req = test::TestRequest::post()
            .uri("/get-service-auth")
            .set_json(&json!({}))
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::NOT_FOUND);
    }

    #[actix_rt::test]
    async fn test_invalid_route() {
        let app_config = create_test_config();

        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(app_config))
                .service(health_check),
        )
        .await;

        let req = test::TestRequest::get().uri("/non-existent").to_request();

        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::NOT_FOUND);
    }

    #[actix_rt::test]
    async fn test_wrong_http_method() {
        let app_config = create_test_config();

        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(app_config))
                .service(create_account_api),
        )
        .await;

        let req = test::TestRequest::get().uri("/create-account").to_request();

        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::NOT_FOUND);
    }

    #[actix_rt::test]
    async fn test_malformed_json() {
        let app_config = create_test_config();

        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(app_config))
                .service(create_account_api),
        )
        .await;

        let req = test::TestRequest::post()
            .uri("/create-account")
            .set_payload("invalid json{")
            .insert_header(("content-type", "application/json"))
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
    }

    #[actix_rt::test]
    async fn test_create_account_with_invalid_did() {
        let app_config = create_test_config();

        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(app_config))
                .service(create_account_api),
        )
        .await;

        let invalid_request = json!({
            "email": "test@example.com",
            "handle": "test.bsky.social",
            "invite_code": "test-invite",
            "password": "testpass123",
            "token": "test-token",
            "pds_host": "https://test.pds.host",
            "did": "invalid-did-format"
        });

        let req = test::TestRequest::post()
            .uri("/create-account")
            .set_json(&invalid_request)
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
    }
}
