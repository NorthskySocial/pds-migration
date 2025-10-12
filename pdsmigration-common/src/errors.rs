use derive_more::{Display, Error};

#[derive(Debug, Display, Error)]
pub enum MigrationError {
    #[display("Validation error on field: {field}")]
    Validation { field: String },
    #[display("Upstream error: {message}")]
    Upstream { message: String },
    #[display("Unexpected error occurred: {message}")]
    Runtime { message: String },
    #[display("Rate limit reached. Please try again later.")]
    RateLimitReached,
}
