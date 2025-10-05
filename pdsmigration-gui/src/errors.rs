use std::fmt::Display;

#[derive(Debug, Clone)]
pub enum GuiError {
    NoMissingBlobs,
    InvalidPdsEndpoint,
    InvalidLogin,
    Runtime,
    Other,
    Success,
    Custom(String),
    AuthFactorTokenRequired,
}

impl Display for GuiError {
    fn fmt(&self, __derive_more_f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::NoMissingBlobs => __derive_more_f.write_fmt(format_args!("No Missing Blobs",)),
            Self::InvalidPdsEndpoint => {
                __derive_more_f.write_fmt(format_args!("Invalid PDS Endpoint",))
            }
            Self::InvalidLogin => {
                __derive_more_f.write_fmt(format_args!("Invalid Username/Password",))
            }
            Self::Runtime => __derive_more_f.write_fmt(format_args!("Runtime Exception",)),
            Self::Other => __derive_more_f.write_fmt(format_args!("Other Exception",)),
            Self::Success => __derive_more_f.write_fmt(format_args!("Success",)),
            Self::Custom(_0) => __derive_more_f.write_fmt(format_args!("{_0}",)),
            Self::AuthFactorTokenRequired => {
                __derive_more_f.write_fmt(format_args!("Auth Factor Token Required",))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gui_error_variants() {
        let errors = vec![
            GuiError::NoMissingBlobs,
            GuiError::InvalidPdsEndpoint,
            GuiError::InvalidLogin,
            GuiError::Runtime,
            GuiError::Other,
            GuiError::Success,
            GuiError::Custom("custom message".to_string()),
            GuiError::AuthFactorTokenRequired,
        ];

        // Test that all error variants can be created
        assert_eq!(errors.len(), 8);

        // Test debug formatting
        assert!(format!("{:?}", GuiError::NoMissingBlobs).contains("NoMissingBlobs"));
        assert!(format!("{:?}", GuiError::Runtime).contains("Runtime"));
    }

    #[test]
    fn test_gui_error_display() {
        // Test Display trait implementation
        assert_eq!(format!("{}", GuiError::NoMissingBlobs), "No Missing Blobs");
        assert_eq!(
            format!("{}", GuiError::InvalidPdsEndpoint),
            "Invalid PDS Endpoint"
        );
        assert_eq!(
            format!("{}", GuiError::InvalidLogin),
            "Invalid Username/Password"
        );
        assert_eq!(format!("{}", GuiError::Runtime), "Runtime Exception");
        assert_eq!(format!("{}", GuiError::Other), "Other Exception");
        assert_eq!(format!("{}", GuiError::Success), "Success");
        assert_eq!(
            format!("{}", GuiError::AuthFactorTokenRequired),
            "Auth Factor Token Required"
        );

        // Test custom error
        let custom_error = GuiError::Custom("Test custom error".to_string());
        assert_eq!(format!("{}", custom_error), "Test custom error");
    }

    #[test]
    fn test_gui_error_clone() {
        let original = GuiError::InvalidLogin;
        let cloned = original.clone();

        // Test that clone works and produces equivalent error
        assert_eq!(format!("{}", original), format!("{}", cloned));
        assert_eq!(format!("{:?}", original), format!("{:?}", cloned));
    }

    #[test]
    fn test_gui_error_custom_variants() {
        let custom1 = GuiError::Custom("Error message 1".to_string());
        let custom2 = GuiError::Custom("Error message 2".to_string());

        assert_eq!(format!("{}", custom1), "Error message 1");
        assert_eq!(format!("{}", custom2), "Error message 2");

        // Test that custom errors with different messages are different
        assert_ne!(format!("{}", custom1), format!("{}", custom2));
    }

    #[test]
    fn test_gui_error_equality_by_discriminant() {
        // Test that errors of the same variant have the same discriminant
        assert_eq!(
            std::mem::discriminant(&GuiError::Runtime),
            std::mem::discriminant(&GuiError::Runtime)
        );
        assert_ne!(
            std::mem::discriminant(&GuiError::Runtime),
            std::mem::discriminant(&GuiError::InvalidLogin)
        );

        // Custom errors should have the same discriminant regardless of message
        assert_eq!(
            std::mem::discriminant(&GuiError::Custom("msg1".to_string())),
            std::mem::discriminant(&GuiError::Custom("msg2".to_string()))
        );
    }
}
