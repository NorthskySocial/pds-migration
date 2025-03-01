use actix_web::http::StatusCode;
use actix_web::{HttpResponse, ResponseError};
use pdsmigration_common::error_code::CustomErrorType;

#[derive(Debug)]
pub struct CustomError {
    pub message: Option<String>,
    pub err_type: CustomErrorType,
}

impl CustomError {
    pub fn message(&self) -> String {
        match &self.message {
            Some(c) => c.clone(),
            None => String::from(""),
        }
    }
}

impl From<pdsmigration_common::error_code::CustomError> for CustomError {
    fn from(value: pdsmigration_common::error_code::CustomError) -> Self {
        Self {
            message: value.message,
            err_type: value.err_type,
        }
    }
} 

impl std::fmt::Display for CustomError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl ResponseError for CustomError {
    fn status_code(&self) -> StatusCode {
        match self.err_type {
            CustomErrorType::ValidationError => StatusCode::BAD_REQUEST,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
    fn error_response(&self) -> HttpResponse {
        HttpResponse::build(self.status_code()).json(self.message.clone())
    }
}

