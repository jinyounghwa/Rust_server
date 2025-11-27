/// JWT Authentication Middleware
///
/// Validates JWT tokens from the Authorization header and injects
/// claims into request extensions for use by route handlers.

use actix_web::{
    dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform},
    Error, HttpMessage, HttpResponse,
};
use futures::future::LocalBoxFuture;
use std::rc::Rc;

use crate::auth::validate_access_token;
use crate::configuration::JwtSettings;

/// JWT middleware for protecting routes
///
/// Must be applied to routes that require authentication.
/// Extracts and validates JWT from Authorization header.
pub struct JwtMiddleware {
    jwt_config: JwtSettings,
}

impl JwtMiddleware {
    /// Create new JWT middleware instance
    pub fn new(jwt_config: JwtSettings) -> Self {
        Self { jwt_config }
    }
}

impl<S, B> Transform<S, ServiceRequest> for JwtMiddleware
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type InitError = ();
    type Transform = JwtMiddlewareService<S>;
    type Future = std::future::Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        std::future::ready(Ok(JwtMiddlewareService {
            service: Rc::new(service),
            jwt_config: self.jwt_config.clone(),
        }))
    }
}

pub struct JwtMiddlewareService<S> {
    service: Rc<S>,
    jwt_config: JwtSettings,
}

impl<S, B> Service<ServiceRequest> for JwtMiddlewareService<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        // Extract Authorization header
        let auth_header = req
            .headers()
            .get("Authorization")
            .and_then(|h| h.to_str().ok())
            .and_then(|h| {
                if h.starts_with("Bearer ") {
                    Some(h[7..].to_string())
                } else {
                    None
                }
            });

        let jwt_config = self.jwt_config.clone();

        match auth_header {
            None => {
                tracing::warn!("Missing or invalid Authorization header");
                let response = HttpResponse::Unauthorized().json(serde_json::json!({
                    "error": "Missing or invalid authorization header",
                    "code": "UNAUTHORIZED"
                }));
                Box::pin(async move {
                    Err(actix_web::error::InternalError::from_response(
                        "Unauthorized",
                        response,
                    )
                    .into())
                })
            }
            Some(token) => {
                match validate_access_token(&token, &jwt_config) {
                    Ok(claims) => {
                        // Inject claims into request extensions
                        req.extensions_mut().insert(claims.clone());

                        tracing::debug!(
                            user_id = %claims.sub,
                            email = %claims.email,
                            "JWT validated successfully"
                        );

                        let service = self.service.clone();
                        Box::pin(async move { service.call(req).await })
                    }
                    Err(e) => {
                        tracing::warn!("JWT validation failed: {}", e);
                        let response = HttpResponse::Unauthorized().json(serde_json::json!({
                            "error": "Invalid or expired token",
                            "code": "TOKEN_INVALID"
                        }));
                        Box::pin(async move {
                            Err(actix_web::error::InternalError::from_response(
                                "Invalid token",
                                response,
                            )
                            .into())
                        })
                    }
                }
            }
        }
    }
}
