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
