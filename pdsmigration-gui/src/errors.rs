use pdsmigration_common::MigrationError;
use std::fmt::Display;

#[derive(Debug, Clone)]
pub enum GuiError {
    NoMissingBlobs,
    InvalidPdsEndpoint,
    InvalidLogin,
    Runtime,
    Other,
    Success,
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
            Self::AuthFactorTokenRequired => {
                __derive_more_f.write_fmt(format_args!("Auth Factor Token Required",))
            }
        }
    }
}
