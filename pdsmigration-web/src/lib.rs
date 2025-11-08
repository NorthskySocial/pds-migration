pub mod api;
pub mod background_jobs;
pub mod config;
pub mod errors;

pub use actix_web::web::Json;
pub use actix_web::{post, HttpResponse};

pub const APPLICATION_JSON: &str = "application/json";
