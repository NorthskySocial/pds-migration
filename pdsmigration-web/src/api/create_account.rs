use crate::errors::{ApiError, ApiErrorBody};
use crate::post;
use actix_web::web::Json;
use actix_web::HttpResponse;
use pdsmigration_common::{create_account, CreateAccountRequest};
use serde::{Deserialize, Serialize};
use std::fmt;
use utoipa::ToSchema;

#[derive(Deserialize, Serialize, ToSchema)]
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

impl fmt::Debug for CreateAccountApiRequest {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("CreateAccountApiRequest")
            .field("email", &self.email)
            .field("handle", &self.handle)
            .field("invite_code", &"[REDACTED]")
            .field("password", &"[REDACTED]")
            .field("token", &"[REDACTED]")
            .field("pds_host", &self.pds_host)
            .field("did", &self.did)
            .field(
                "recovery_key",
                &self.recovery_key.as_ref().map(|_| "[REDACTED]"),
            )
            .finish()
    }
}

#[utoipa::path(
    post,
    path = "/create-account",
    request_body = CreateAccountApiRequest,
    responses(
        (status = 200, description = "Account created successfully"),
        (status = 400, description = "Invalid request", body = ApiErrorBody, content_type = "application/json")
    ),
    tag = "pdsmigration-web"
)]
#[tracing::instrument(skip(req), fields(
    email = %req.email,
    handle = %req.handle,
    pds_host = %req.pds_host,
    did = %req.did
))]
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
