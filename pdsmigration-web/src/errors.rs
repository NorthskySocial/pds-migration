use actix_web::http::StatusCode;
use actix_web::{HttpResponse, ResponseError};
use pdsmigration_common::errors::PdsError;

#[derive(Debug)]
pub enum ApiError {
    Validation,
    AccountStatus,
    Login,
    Runtime,
    CreateAccount,
    AccountExport,
    AccountImport,
}

impl From<PdsError> for ApiError {
    fn from(value: PdsError) -> Self {
        match value {
            PdsError::Validation => ApiError::Validation,
            PdsError::AccountStatus => ApiError::AccountStatus,
            PdsError::Login => ApiError::Login,
            PdsError::Runtime => ApiError::Runtime,
            PdsError::CreateAccount => ApiError::CreateAccount,
            PdsError::AccountExport => ApiError::AccountExport,
            PdsError::AccountImport => ApiError::AccountImport,
            PdsError::RateLimitReached => ApiError::Runtime,
        }
    }
}

impl std::fmt::Display for ApiError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}

impl ResponseError for ApiError {
    fn status_code(&self) -> StatusCode {
        match *self {
            ApiError::Validation => StatusCode::BAD_REQUEST,
            ApiError::AccountStatus => StatusCode::BAD_REQUEST,
            ApiError::Login => StatusCode::BAD_REQUEST,
            ApiError::Runtime => StatusCode::INTERNAL_SERVER_ERROR,
            ApiError::CreateAccount => StatusCode::BAD_REQUEST,
            ApiError::AccountExport => StatusCode::BAD_REQUEST,
            ApiError::AccountImport => StatusCode::BAD_REQUEST,
        }
    }
    fn error_response(&self) -> HttpResponse {
        HttpResponse::build(self.status_code()).json(())
    }
}
