use actix_web::http::header::ContentType;
use actix_web::http::StatusCode;
use actix_web::{HttpResponse, ResponseError};
use derive_more::{Display, Error};
use pdsmigration_common::MigrationError;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ApiErrorBody {
    code: String,
    message: String,
}

#[derive(Debug, Display, Error)]
pub enum ApiError {
    #[display("Validation error on field: {field}")]
    Validation { field: String },
    #[display("Upstream error: {message}")]
    Upstream { message: String },
    #[display("Unexpected error occurred: {message}")]
    Runtime { message: String },
    #[display("Authentication error: {message}")]
    Authentication { message: String },
}

impl ResponseError for ApiError {
    fn status_code(&self) -> StatusCode {
        match self {
            ApiError::Validation { .. } => StatusCode::BAD_REQUEST,
            ApiError::Upstream { .. } => StatusCode::INTERNAL_SERVER_ERROR,
            ApiError::Runtime { .. } => StatusCode::INTERNAL_SERVER_ERROR,
            ApiError::Authentication { .. } => StatusCode::UNAUTHORIZED,
        }
    }

    fn error_response(&self) -> HttpResponse {
        let (code, message) = match self {
            ApiError::Validation { field } => ("VALIDATION_ERROR", field.to_string()),
            ApiError::Upstream { message } => ("UPSTREAM_ERROR", message.to_string()),
            ApiError::Runtime { message } => ("RUNTIME_ERROR", message.to_string()),
            ApiError::Authentication { message } => ("AUTHENTICATION_ERROR", message.to_string()),
        };

        HttpResponse::build(self.status_code())
            .insert_header(ContentType::json())
            .json(ApiErrorBody {
                code: code.to_string(),
                message,
            })
    }
}

impl From<MigrationError> for ApiError {
    fn from(error: MigrationError) -> Self {
        match error {
            MigrationError::Validation { field } => ApiError::Validation { field },
            MigrationError::Upstream { message } => ApiError::Upstream { message },
            MigrationError::Runtime { message } => ApiError::Runtime { message },
            MigrationError::RateLimitReached => ApiError::Runtime {
                message: "Rate limit reached. Please try again later.".to_string(),
            },
            MigrationError::Authentication { .. } => ApiError::Authentication {
                message: "Authentication failed".to_string(),
            },
        }
    }
}
