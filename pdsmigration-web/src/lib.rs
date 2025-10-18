pub mod api;
pub mod config;
pub mod errors;

pub use actix_web::web::Json;
pub use actix_web::{post, HttpResponse};

pub const APPLICATION_JSON: &str = "application/json";
