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

#[cfg(test)]
mod tests {
    use super::*;
    use actix_web::http::StatusCode;
    use pdsmigration_common::errors::PdsError;
    use pretty_assertions::assert_eq;

    #[test]
    fn test_api_error_display() {
        assert_eq!(format!("{}", ApiError::Validation), "Validation");
        assert_eq!(format!("{}", ApiError::AccountStatus), "AccountStatus");
        assert_eq!(format!("{}", ApiError::Login), "Login");
        assert_eq!(format!("{}", ApiError::Runtime), "Runtime");
        assert_eq!(format!("{}", ApiError::CreateAccount), "CreateAccount");
        assert_eq!(format!("{}", ApiError::AccountExport), "AccountExport");
        assert_eq!(format!("{}", ApiError::AccountImport), "AccountImport");
    }

    #[test]
    fn test_api_error_status_codes() {
        assert_eq!(ApiError::Validation.status_code(), StatusCode::BAD_REQUEST);
        assert_eq!(
            ApiError::AccountStatus.status_code(),
            StatusCode::BAD_REQUEST
        );
        assert_eq!(ApiError::Login.status_code(), StatusCode::BAD_REQUEST);
        assert_eq!(
            ApiError::Runtime.status_code(),
            StatusCode::INTERNAL_SERVER_ERROR
        );
        assert_eq!(
            ApiError::CreateAccount.status_code(),
            StatusCode::BAD_REQUEST
        );
        assert_eq!(
            ApiError::AccountExport.status_code(),
            StatusCode::BAD_REQUEST
        );
        assert_eq!(
            ApiError::AccountImport.status_code(),
            StatusCode::BAD_REQUEST
        );
    }

    #[test]
    fn test_api_error_from_pds_error() {
        let api_error: ApiError = PdsError::Validation.into();
        assert!(matches!(api_error, ApiError::Validation));

        let api_error: ApiError = PdsError::AccountStatus.into();
        assert!(matches!(api_error, ApiError::AccountStatus));

        let api_error: ApiError = PdsError::Login.into();
        assert!(matches!(api_error, ApiError::Login));

        let api_error: ApiError = PdsError::Runtime.into();
        assert!(matches!(api_error, ApiError::Runtime));

        let api_error: ApiError = PdsError::CreateAccount.into();
        assert!(matches!(api_error, ApiError::CreateAccount));

        let api_error: ApiError = PdsError::AccountExport.into();
        assert!(matches!(api_error, ApiError::AccountExport));

        let api_error: ApiError = PdsError::AccountImport.into();
        assert!(matches!(api_error, ApiError::AccountImport));

        // RateLimitReached should map to Runtime
        let api_error: ApiError = PdsError::RateLimitReached.into();
        assert!(matches!(api_error, ApiError::Runtime));
    }

    #[test]
    fn test_api_error_response() {
        let validation_error = ApiError::Validation;
        let response = validation_error.error_response();
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);

        let runtime_error = ApiError::Runtime;
        let response = runtime_error.error_response();
        assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
    }

    #[test]
    fn test_api_error_debug() {
        // Test that Debug is properly implemented
        assert_eq!(format!("{:?}", ApiError::Validation), "Validation");
        assert_eq!(format!("{:?}", ApiError::Runtime), "Runtime");
    }
}
