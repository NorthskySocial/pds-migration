use actix_web::http::header::ContentType;
use actix_web::http::StatusCode;
use actix_web::{HttpResponse, ResponseError};
use derive_more::{Display, Error};
use pdsmigration_common::MigrationError;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Display, Clone, Serialize, Deserialize, ToSchema)]
pub enum ApiErrorCode {
    #[display("VALIDATION_ERROR")]
    Validation,
    #[display("UPSTREAM_ERROR")]
    Upstream,
    #[display("RUNTIME_ERROR")]
    Runtime,
    #[display("AUTHENTICATION_ERROR")]
    Authentication,
    #[display("RATE_LIMIT")]
    RateLimit,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ApiErrorBody {
    #[schema(example = ApiErrorCode::Validation)]
    code: ApiErrorCode,
    #[schema(example = "did")]
    message: String,
}

#[derive(Debug, Display, Error, ToSchema)]
pub enum ApiError {
    #[display("Validation error on field: {field}")]
    #[schema(title = "Validation")]
    Validation { field: String },
    #[display("Upstream error: {message}")]
    #[schema(title = "Upstream")]
    Upstream { message: String },
    #[display("Unexpected error occurred: {message}")]
    #[schema(title = "Runtime")]
    Runtime { message: String },
    #[display("Authentication error: {message}")]
    #[schema(title = "Authentication")]
    Authentication { message: String },
    #[display("Too many requests: {message}")]
    #[schema(title = "Rate limit")]
    RateLimit { message: String },
}

impl ResponseError for ApiError {
    fn status_code(&self) -> StatusCode {
        match self {
            ApiError::Validation { .. } => StatusCode::BAD_REQUEST,
            ApiError::Upstream { .. } => StatusCode::INTERNAL_SERVER_ERROR,
            ApiError::Runtime { .. } => StatusCode::INTERNAL_SERVER_ERROR,
            ApiError::Authentication { .. } => StatusCode::UNAUTHORIZED,
            ApiError::RateLimit { .. } => StatusCode::TOO_MANY_REQUESTS,
        }
    }

    fn error_response(&self) -> HttpResponse {
        let (code, message) = match self {
            ApiError::Validation { field } => (ApiErrorCode::Validation, field.to_string()),
            ApiError::Upstream { message } => (ApiErrorCode::Upstream, message.to_string()),
            ApiError::Runtime { message } => (ApiErrorCode::Runtime, message.to_string()),
            ApiError::Authentication { message } => {
                (ApiErrorCode::Authentication, message.to_string())
            }
            ApiError::RateLimit { message } => (ApiErrorCode::RateLimit, message.to_string()),
        };

        HttpResponse::build(self.status_code())
            .insert_header(ContentType::json())
            .json(ApiErrorBody { code, message })
    }
}

impl From<MigrationError> for ApiError {
    fn from(error: MigrationError) -> Self {
        match error {
            MigrationError::Validation { field } => ApiError::Validation { field },
            MigrationError::Upstream { message } => ApiError::Upstream { message },
            MigrationError::Runtime { message } => ApiError::Runtime { message },
            MigrationError::RateLimitReached => ApiError::RateLimit {
                message: "Rate limit reached. Please try again later.".to_string(),
            },
            MigrationError::Authentication { .. } => ApiError::Authentication {
                message: "Authentication failed".to_string(),
            },
        }
    }
}
