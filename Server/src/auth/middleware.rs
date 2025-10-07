use axum::{
    extract::{Request, State},
    http::{header, StatusCode},
    middleware::Next,
    response::Response,
};
use std::sync::Arc;

use crate::auth::jwt::JwtService;
use crate::api::response::ApiError;

#[derive(Clone)]
pub struct AuthState {
    pub jwt_service: Arc<JwtService>,
}

/// Middleware to require authentication for a route
pub async fn require_auth(
    State(state): State<AuthState>,
    mut request: Request,
    next: Next,
) -> Result<Response, ApiError> {
    // Extract token from Authorization header
    let token = request
        .headers()
        .get(header::AUTHORIZATION)
        .and_then(|value| value.to_str().ok())
        .and_then(|value| {
            if value.starts_with("Bearer ") {
                Some(&value[7..])
            } else {
                None
            }
        })
        .ok_or_else(|| {
            ApiError::new(
                StatusCode::UNAUTHORIZED,
                "Missing or invalid Authorization header",
            )
        })?;

    // Verify token
    let claims = state
        .jwt_service
        .verify_token(token)
        .map_err(|_| {
            ApiError::new(StatusCode::UNAUTHORIZED, "Invalid or expired token")
        })?;

    // Add claims to request extensions
    request.extensions_mut().insert(claims);

    Ok(next.run(request).await)
}

/// Middleware to require admin authentication for a route
pub async fn require_admin(
    State(state): State<AuthState>,
    mut request: Request,
    next: Next,
) -> Result<Response, ApiError> {
    // Extract token from Authorization header
    let token = request
        .headers()
        .get(header::AUTHORIZATION)
        .and_then(|value| value.to_str().ok())
        .and_then(|value| {
            if value.starts_with("Bearer ") {
                Some(&value[7..])
            } else {
                None
            }
        })
        .ok_or_else(|| {
            ApiError::new(
                StatusCode::UNAUTHORIZED,
                "Missing or invalid Authorization header",
            )
        })?;

    // Verify token
    let claims = state
        .jwt_service
        .verify_token(token)
        .map_err(|_| {
            ApiError::new(StatusCode::UNAUTHORIZED, "Invalid or expired token")
        })?;

    // Check if user is admin
    if !claims.is_admin {
        return Err(ApiError::new(
            StatusCode::FORBIDDEN,
            "Admin access required",
        ));
    }

    // Add claims to request extensions
    request.extensions_mut().insert(claims);

    Ok(next.run(request).await)
}
