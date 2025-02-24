use crate::agent::{
    account_export, account_import, activate_account, create_account, deactivate_account,
    export_preferences, get_blob, get_service_auth, import_preferences, missing_blobs,
    recommended_plc, request_token, sign_plc, submit_plc, upload_blob,
};
use crate::error_code::{CustomError, CustomErrorType};
use actix_web::dev::Server;
use actix_web::web::Json;
use actix_web::{middleware, post, App, HttpResponse, HttpServer};
use bsky_sdk::api::agent::atp_agent::AtpSession;
use bsky_sdk::api::agent::Configure;
use bsky_sdk::api::types::string::Did;
use bsky_sdk::BskyAgent;
use dotenvy::dotenv;
use ipld_core::cid::Cid;
use serde::{Deserialize, Serialize};
use std::{env, io};
use std::io::ErrorKind;

pub mod agent;
pub mod error_code;

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
    .bind(format!("0.0.0.0:{}", server_port))
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

#[derive(Debug, Deserialize, Serialize)]
pub struct ServiceAuthRequest {
    pub pds_host: String,
    pub aud: String,
    pub handle: String,
    pub password: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ServiceAuthResponse {
    pub token: String,
}

#[post("/service-auth")]
pub async fn get_service_auth_api(
    req: Json<ServiceAuthRequest>,
) -> Result<HttpResponse, CustomError> {
    let agent = BskyAgent::builder().build().await.unwrap();
    login_helper(
        &agent,
        req.pds_host.as_str(),
        req.handle.as_str(),
        req.password.as_str(),
    )
    .await?;
    let token = get_service_auth(&agent, req.aud.as_str()).await?;
    let response = ServiceAuthResponse { token };
    Ok(HttpResponse::Ok().json(response))
}

#[derive(Debug, Deserialize, Serialize)]
pub struct CreateAccountApiRequest {
    pub email: String,
    pub handle: String,
    pub invite_code: String,
    pub password: String,
    pub token: String,
    pub pds_host: String,
    pub did: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct CreateAccountRequest {
    pub did: Did,
    pub email: Option<String>,
    pub handle: String,
    pub invite_code: Option<String>,
    pub password: Option<String>,
    pub recovery_key: Option<String>,
    pub verification_code: Option<String>,
    pub verification_phone: Option<String>,
    pub plc_op: Option<String>,
    pub token: String,
}

#[tracing::instrument(skip(req))]
#[post("/create-account")]
pub async fn create_account_api(
    req: Json<CreateAccountApiRequest>,
) -> Result<HttpResponse, CustomError> {
    create_account(
        req.pds_host.as_str(),
        &CreateAccountRequest {
            did: req.did.parse().unwrap(),
            email: Some(req.email.clone()),
            handle: req.handle.parse().unwrap(),
            invite_code: Some(req.invite_code.clone()),
            password: Some(req.password.clone()),
            recovery_key: None,
            verification_code: Some(String::from("")),
            verification_phone: None,
            plc_op: None,
            token: req.token.clone(),
        },
    )
    .await?;
    Ok(HttpResponse::Ok().finish())
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ExportPDSRequest {
    pub pds_host: String,
    pub handle: String,
    pub password: String,
}

#[tracing::instrument]
#[post("/export-repo")]
pub async fn export_pds_api(req: Json<ExportPDSRequest>) -> Result<HttpResponse, CustomError> {
    let agent = BskyAgent::builder().build().await.unwrap();
    let session = login_helper(
        &agent,
        req.pds_host.as_str(),
        req.handle.as_str(),
        req.password.as_str(),
    )
    .await?;
    account_export(&agent, &session.did).await?;
    Ok(HttpResponse::Ok().finish())
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ImportPDSRequest {
    pub pds_host: String,
    pub handle: String,
    pub password: String,
}

#[tracing::instrument]
#[post("/import-repo")]
pub async fn import_pds_api(req: Json<ImportPDSRequest>) -> Result<HttpResponse, CustomError> {
    let agent = BskyAgent::builder().build().await.unwrap();
    let session = login_helper(
        &agent,
        req.pds_host.as_str(),
        req.handle.as_str(),
        req.password.as_str(),
    )
    .await?;
    let endpoint_url = env::var("ENDPOINT").unwrap();
    let config = aws_config::from_env().region("auto").endpoint_url(endpoint_url).load().await;
    let client = aws_sdk_s3::Client::new(&config);
    let bucket_name = "migration".to_string();
    let file_name = session.did.as_str().to_string() + ".car";
    let key = "migration".to_string() + session.did.as_str() + ".car";
    let body = aws_sdk_s3::primitives::ByteStream::from_path(std::path::Path::new(file_name.as_str())).await;
    match client.put_object().bucket(&bucket_name).key(&key).body(body.unwrap()).send().await {
        Ok(output) => {
            tracing::info!("{:?}", output);
        }
        Err(e) => {
            tracing::error!("{:?}", e);
            return Err(CustomError {
                message: None,
                err_type: CustomErrorType::ValidationError,
            })
        }
    }
    account_import(&agent, (session.did.as_str().to_string() + ".car").as_str()).await?;
    Ok(HttpResponse::Ok().content_type(APPLICATION_JSON).finish())
}

#[derive(Debug, Deserialize, Serialize)]
pub struct MissingBlobsRequest {
    pub pds_host: String,
    pub handle: String,
    pub password: String,
}

#[tracing::instrument]
#[post("/missing-blobs")]
pub async fn missing_blobs_api(
    req: Json<MissingBlobsRequest>,
) -> Result<HttpResponse, CustomError> {
    let agent = BskyAgent::builder().build().await.unwrap();
    login_helper(
        &agent,
        req.pds_host.as_str(),
        req.handle.as_str(),
        req.password.as_str(),
    )
    .await?;
    let initial_missing_blobs = missing_blobs(&agent).await?;
    let mut missing_blob_cids = Vec::new();
    for blob in &initial_missing_blobs {
        missing_blob_cids.push(Cid::to_string(blob.cid.as_ref()));
    }

    let response = serde_json::to_string(&missing_blob_cids).unwrap();
    Ok(HttpResponse::Ok()
        .content_type(APPLICATION_JSON)
        .body(response))
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ExportBlobsRequest {
    pub new_pds_host: String,
    pub new_handle: String,
    pub new_password: String,
    pub old_pds_host: String,
    pub old_handle: String,
    pub old_password: String,
}

#[tracing::instrument]
#[post("/export-blobs")]
pub async fn export_blobs_api(req: Json<ExportBlobsRequest>) -> Result<HttpResponse, CustomError> {
    let agent = BskyAgent::builder().build().await.unwrap();
    login_helper(
        &agent,
        req.new_pds_host.as_str(),
        req.new_handle.as_str(),
        req.new_password.as_str(),
    )
    .await?;
    let missing_blobs = missing_blobs(&agent).await?;
    let session = login_helper(
        &agent,
        req.old_pds_host.as_str(),
        req.old_handle.as_str(),
        req.old_password.as_str(),
    )
    .await?;
    for missing_blob in &missing_blobs {
        match std::fs::create_dir(session.did.as_str()) {
            Ok(_) => {}
            Err(e) => {
                if e.kind() != ErrorKind::AlreadyExists {
                    tracing::error!("Error creating directory: {:?}", e);
                    return Err(CustomError {
                        message: None,
                        err_type: CustomErrorType::ValidationError,
                    });
                }
            }
        }
        match get_blob(&agent, missing_blob.cid.clone(), session.did.clone()).await {
            Ok(output) => {
                tracing::info!("Successfully fetched missing blob");
                std::fs::write(
                    String::from(session.did.as_str())
                        + "/"
                        + missing_blob.record_uri.as_str().split("/").last().unwrap(),
                    output,
                )
                .unwrap();
            }
            Err(_) => {
                tracing::error!("Failed to determine missing blobs");
                return Err(CustomError {
                    message: None,
                    err_type: CustomErrorType::ValidationError,
                });
            }
        }
    }

    Ok(HttpResponse::Ok().content_type(APPLICATION_JSON).finish())
}

#[derive(Debug, Deserialize, Serialize)]
pub struct UploadBlobsRequest {
    pub pds_host: String,
    pub handle: String,
    pub password: String,
}

#[tracing::instrument]
#[post("/upload-blobs")]
pub async fn upload_blobs_api(req: Json<UploadBlobsRequest>) -> Result<HttpResponse, CustomError> {
    let agent = BskyAgent::builder().build().await.unwrap();
    agent.configure_endpoint(req.pds_host.clone());
    let session = login_helper(
        &agent,
        req.pds_host.as_str(),
        req.handle.as_str(),
        req.password.as_str(),
    )
    .await?;

    let blob_dir;
    match std::fs::read_dir(session.did.as_str()) {
        Ok(output) => blob_dir = output,
        Err(_) => {
            return Err(CustomError {
                message: None,
                err_type: CustomErrorType::ValidationError,
            })
        }
    }
    for blob in blob_dir {
        let file = std::fs::read(blob.unwrap().path()).unwrap();
        upload_blob(&agent, file).await?;
    }

    Ok(HttpResponse::Ok().content_type(APPLICATION_JSON).finish())
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ActivateAccountRequest {
    pub pds_host: String,
    pub handle: String,
    pub password: String,
}

#[tracing::instrument]
#[post("/activate-account")]
pub async fn activate_account_api(
    req: Json<ActivateAccountRequest>,
) -> Result<HttpResponse, CustomError> {
    let agent = BskyAgent::builder().build().await.unwrap();
    login_helper(
        &agent,
        req.pds_host.as_str(),
        req.handle.as_str(),
        req.password.as_str(),
    )
    .await?;
    activate_account(&agent).await?;
    Ok(HttpResponse::Ok().content_type(APPLICATION_JSON).finish())
}

#[derive(Debug, Deserialize, Serialize)]
pub struct DeactivateAccountRequest {
    pub pds_host: String,
    pub handle: String,
    pub password: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct DeactivateAccountResponse {}

#[tracing::instrument]
#[post("/deactivate-account")]
pub async fn deactivate_account_api(
    req: Json<DeactivateAccountRequest>,
) -> Result<HttpResponse, CustomError> {
    let agent = BskyAgent::builder().build().await.unwrap();
    login_helper(
        &agent,
        req.pds_host.as_str(),
        req.handle.as_str(),
        req.password.as_str(),
    )
    .await?;
    deactivate_account(&agent).await?;
    Ok(HttpResponse::Ok().content_type(APPLICATION_JSON).finish())
}

#[derive(Debug, Deserialize, Serialize)]
pub struct MigratePreferencesRequest {
    pub new_pds_host: String,
    pub new_handle: String,
    pub new_password: String,
    pub old_pds_host: String,
    pub old_handle: String,
    pub old_password: String,
}

#[tracing::instrument]
#[post("/migrate-preferences")]
pub async fn migrate_preferences_api(
    req: Json<MigratePreferencesRequest>,
) -> Result<HttpResponse, CustomError> {
    let agent = BskyAgent::builder().build().await.unwrap();
    login_helper(
        &agent,
        req.old_pds_host.as_str(),
        req.old_handle.as_str(),
        req.old_password.as_str(),
    )
    .await?;
    let preferences = export_preferences(&agent).await?;
    login_helper(
        &agent,
        req.new_pds_host.as_str(),
        req.new_handle.as_str(),
        req.new_password.as_str(),
    )
    .await?;
    import_preferences(&agent, preferences).await?;
    Ok(HttpResponse::Ok().content_type(APPLICATION_JSON).finish())
}

#[derive(Debug, Deserialize, Serialize)]
pub struct RequestTokenRequest {
    pub pds_host: String,
    pub handle: String,
    pub password: String,
}

#[tracing::instrument]
#[post("/request-token")]
pub async fn request_token_api(
    req: Json<RequestTokenRequest>,
) -> Result<HttpResponse, CustomError> {
    let agent = BskyAgent::builder().build().await.unwrap();
    login_helper(
        &agent,
        req.pds_host.as_str(),
        req.handle.as_str(),
        req.password.as_str(),
    )
    .await?;
    request_token(&agent).await?;
    Ok(HttpResponse::Ok().content_type(APPLICATION_JSON).finish())
}

#[derive(Debug, Deserialize, Serialize)]
pub struct MigratePlcRequest {
    pub new_pds_host: String,
    pub new_handle: String,
    pub new_password: String,
    pub old_pds_host: String,
    pub old_handle: String,
    pub old_password: String,
    pub plc_signing_token: String,
}

#[tracing::instrument(skip(req))]
#[post("/migrate-plc")]
pub async fn migrate_plc_api(req: Json<MigratePlcRequest>) -> Result<HttpResponse, CustomError> {
    let agent = BskyAgent::builder().build().await.unwrap();
    login_helper(
        &agent,
        req.new_pds_host.as_str(),
        req.new_handle.as_str(),
        req.new_password.as_str(),
    )
    .await?;
    let recommended_did = recommended_plc(&agent).await?;
    use bsky_sdk::api::com::atproto::identity::sign_plc_operation::InputData;

    let new_plc = InputData {
        also_known_as: recommended_did.also_known_as,
        rotation_keys: recommended_did.rotation_keys,
        services: recommended_did.services,
        token: Some(req.plc_signing_token.clone()),
        verification_methods: recommended_did.verification_methods,
    };
    login_helper(
        &agent,
        req.old_pds_host.as_str(),
        req.old_handle.as_str(),
        req.old_password.as_str(),
    )
    .await?;
    let output = sign_plc(&agent, new_plc.clone()).await?;
    login_helper(
        &agent,
        req.new_pds_host.as_str(),
        req.new_handle.as_str(),
        req.new_password.as_str(),
    )
    .await?;
    submit_plc(&agent, output).await?;

    Ok(HttpResponse::Ok().content_type(APPLICATION_JSON).finish())
}

#[tracing::instrument(skip(password, agent))]
async fn login_helper(
    agent: &BskyAgent,
    pds_host: &str,
    username: &str,
    password: &str,
) -> Result<AtpSession, CustomError> {
    agent.configure_endpoint(pds_host.to_string());
    match agent.login(username, password).await {
        Ok(session) => {
            tracing::info!("Successfully logged in");
            Ok(session)
        }
        Err(e) => {
            tracing::error!("Error logging in: {:?}", e);
            Err(CustomError {
                message: Some("Failed to login".to_string()),
                err_type: CustomErrorType::LoginError,
            })
        }
    }
}
