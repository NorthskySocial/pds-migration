use derive_more::{Display, Error};

#[derive(Debug, Display, Error, Clone)]
pub enum GuiError {
    #[display("No Missing Blobs")]
    NoMissingBlobs,
    #[display("Invalid PDS Endpoint")]
    InvalidPdsEndpoint,
    #[display("Invalid Username/Password")]
    InvalidLogin,
    #[display("Runtime Exception")]
    Runtime,
    #[display("Other Exception")]
    Other,
    #[display("Success")]
    Success,
}
