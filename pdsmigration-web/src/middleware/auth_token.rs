use actix_web::body::{BoxBody, EitherBody};
use actix_web::dev::{Service, ServiceRequest, ServiceResponse, Transform};
use actix_web::http::header::HeaderMap;
use actix_web::{web, Error, ResponseError};
use futures::future::{ready, LocalBoxFuture, Ready};
use std::sync::Arc;
use std::task::{Context, Poll};

use crate::config::AppConfig;
use crate::errors::ApiError;

#[derive(Clone, Default)]
pub struct AuthToken;

impl AuthToken {
    pub fn new() -> Self {
        Self
    }
}

impl<S, B> Transform<S, ServiceRequest> for AuthToken
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
{
    type Response = ServiceResponse<EitherBody<BoxBody, B>>;
    type Error = Error;
    type Transform = AuthTokenMiddleware<S, B>;
    type InitError = ();
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(AuthTokenMiddleware {
            service: Arc::new(service),
            _phantom: std::marker::PhantomData,
        }))
    }
}

pub struct AuthTokenMiddleware<S, B> {
    service: Arc<S>,
    _phantom: std::marker::PhantomData<B>,
}

impl<S, B> Service<ServiceRequest> for AuthTokenMiddleware<S, B>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
{
    type Response = ServiceResponse<EitherBody<BoxBody, B>>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    fn poll_ready(&self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.service.poll_ready(cx)
    }

    fn call(&self, req: ServiceRequest) -> Self::Future {
        // Bypass for certain paths
        let path = req.path();
        let bypass = is_bypass_path(path);

        // Get configured token (if any)
        let maybe_cfg = req.app_data::<web::Data<AppConfig>>().cloned();

        let service = self.service.clone();

        Box::pin(async move {
            let configured_token = maybe_cfg.as_ref().and_then(|d| d.server.auth_token.clone());

            if !bypass {
                if let Some(expected) = configured_token {
                    // Enforce token check
                    let headers = req.headers();
                    if !is_authorized(headers, &expected) {
                        let api_err = ApiError::Authentication {
                            message: "Invalid or missing auth token".to_string(),
                        };
                        let resp = api_err.error_response();
                        return Ok(req.into_response(resp).map_into_left_body());
                    }
                }
            }

            let res = service.call(req).await?;
            Ok(res.map_into_right_body())
        })
    }
}

fn is_bypass_path(path: &str) -> bool {
    // Allow health checks and documentation/metrics without auth
    path == "/health"
        || path.starts_with("/swagger-ui")
        || path.starts_with("/api-docs")
        || path == "/metrics"
}

fn is_authorized(headers: &HeaderMap, expected: &str) -> bool {
    if let Some(h) = headers.get("X-Auth-Token") {
        if let Ok(v) = h.to_str() {
            if v == expected {
                return true;
            }
        }
    }

    false
}
