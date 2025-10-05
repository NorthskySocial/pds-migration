use derive_more::{Display, Error};

#[derive(Debug, Display, Error)]
pub enum PdsError {
    Validation,
    #[display("Error getting account status")]
    AccountStatus,
    Login,
    #[display("Error getting account status")]
    Runtime,
    CreateAccount,
    AccountExport,
    AccountImport,
    RateLimitReached,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_variants() {
        let errors = vec![
            PdsError::Validation,
            PdsError::AccountStatus,
            PdsError::Login,
            PdsError::Runtime,
            PdsError::CreateAccount,
            PdsError::AccountExport,
            PdsError::AccountImport,
            PdsError::RateLimitReached,
        ];

        // Test that all error variants can be created
        assert_eq!(errors.len(), 8);

        // Test debug formatting
        assert!(format!("{:?}", PdsError::Validation).contains("Validation"));
        assert!(format!("{:?}", PdsError::Login).contains("Login"));
    }

    #[test]
    fn test_error_display() {
        // Test Display trait implementation
        assert_eq!(
            format!("{}", PdsError::AccountStatus),
            "Error getting account status"
        );
        assert_eq!(
            format!("{}", PdsError::Runtime),
            "Error getting account status"
        );
        assert_eq!(format!("{}", PdsError::Validation), "Validation");
        assert_eq!(format!("{}", PdsError::Login), "Login");
        assert_eq!(format!("{}", PdsError::CreateAccount), "CreateAccount");
        assert_eq!(format!("{}", PdsError::AccountExport), "AccountExport");
        assert_eq!(format!("{}", PdsError::AccountImport), "AccountImport");
        assert_eq!(
            format!("{}", PdsError::RateLimitReached),
            "RateLimitReached"
        );
    }

    #[test]
    fn test_error_trait() {
        use std::error::Error;

        let error = PdsError::Runtime;
        // Test that it implements std::error::Error
        assert!(error.source().is_none());

        // Test error conversion
        let _: Box<dyn Error> = Box::new(PdsError::Validation);
    }

    #[test]
    fn test_error_equality() {
        // Test that errors of the same variant are equal
        assert_eq!(
            std::mem::discriminant(&PdsError::Validation),
            std::mem::discriminant(&PdsError::Validation)
        );
        assert_ne!(
            std::mem::discriminant(&PdsError::Validation),
            std::mem::discriminant(&PdsError::Login)
        );
    }
}
