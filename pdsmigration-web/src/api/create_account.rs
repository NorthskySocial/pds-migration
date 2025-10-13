use crate::errors::ApiError;
use crate::post;
use actix_web::web::Json;
use actix_web::HttpResponse;
use pdsmigration_common::{create_account, CreateAccountRequest};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct CreateAccountApiRequest {
    pub email: String,
    pub handle: String,
    pub invite_code: String,
    pub password: String,
    pub token: String,
    pub pds_host: String,
    pub did: String,
    #[serde(skip_serializing_if = "core::option::Option::is_none")]
    pub recovery_key: Option<String>,
}

#[tracing::instrument(skip(req))]
#[post("/create-account")]
pub async fn create_account_api(
    req: Json<CreateAccountApiRequest>,
) -> Result<HttpResponse, ApiError> {
    tracing::info!("Create account request received");
    let req = req.into_inner();

    let did = req.did.parse().map_err(|_error| ApiError::Validation {
        field: "did".to_string(),
    })?;

    let handle = req.handle.parse().map_err(|_error| ApiError::Validation {
        field: "handle".to_string(),
    })?;

    create_account(
        req.pds_host.as_str(),
        &CreateAccountRequest {
            did,
            email: Some(req.email.clone()),
            handle,
            invite_code: Some(req.invite_code.trim().to_string()),
            password: Some(req.password.clone()),
            recovery_key: req.recovery_key.clone(),
            verification_code: Some(String::from("")),
            verification_phone: None,
            plc_op: None,
            token: Some(req.token.clone()),
        },
    )
    .await
    .map_err(|e| ApiError::Upstream {
        message: e.to_string(),
    })?;
    tracing::info!("Account created successfully");
    Ok(HttpResponse::Ok().finish())
}
