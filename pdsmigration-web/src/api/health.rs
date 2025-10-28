use crate::HttpResponse;
use actix_web::{get, Responder};
use std::env;

#[utoipa::path(
    get,
    path = "/health",
    responses(
        (status = 200, description = "Health check OK", body = String)
    ),
    tag = "pdsmigration-web"
)]
#[get("/health")]
pub async fn health_check() -> impl Responder {
    HttpResponse::Ok().body("OK")
}

#[utoipa::path(
    get,
    path = "/longhealth",
    responses(
        (status = 200, description = "Health check OK", body = String)
    ),
    tag = "pdsmigration-web"
)]
#[get("/longhealth")]
pub async fn long_health_check() -> impl Responder {
    let time = env::var("TIME_TO_SLEEP")
        .unwrap_or("60".to_string())
        .parse::<u64>()
        .unwrap_or(60);
    tokio::time::sleep(std::time::Duration::from_secs(time)).await;
    HttpResponse::Ok().body("OK")
}
