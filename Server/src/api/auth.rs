use axum::{
    extract::{Json, State, Extension},
    http::StatusCode,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use validator::Validate;

use crate::api::response::{ApiError, ApiResponse, ApiResultNoData};
use crate::auth::{JwtService, PasswordService, Claims};
use crate::db::Database;

// ============================================================================
// Request/Response Types
// ============================================================================

#[derive(Debug, Deserialize, Validate)]
pub struct RegisterRequest {
    #[validate(length(min = 3, max = 20))]
    pub username: String,
    
    #[validate(email)]
    pub email: String,
    
    #[validate(length(min = 8))]
    pub password: String,
    
    pub confirm_password: String,
}

#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Serialize)]
pub struct AuthResponse {
    pub token: String,
    pub is_admin: bool,
}

#[derive(Debug, Serialize)]
pub struct HealthResponse {
    pub status: String,
    pub protocols: ProtocolInfo,
    pub server: String,
    pub version: String,
}

#[derive(Debug, Serialize)]
pub struct ProtocolInfo {
    pub http: bool,
    pub https: bool,
}

// ============================================================================
// Handlers
// ============================================================================

/// GET /api/health
/// Check server health status and available protocols
pub async fn health_check() -> Result<Json<HealthResponse>, ApiError> {
    Ok(Json(HealthResponse {
        status: "OK".to_string(),
        protocols: ProtocolInfo {
            http: true,
            https: true,
        },
        server: "Muse Music Server".to_string(),
        version: "1.0.0".to_string(),
    }))
}

#[derive(Clone)]
pub struct AppState {
    pub db: Arc<dyn Database>,
    pub jwt_service: Arc<JwtService>,
    pub password_service: Arc<PasswordService>,
}

/// POST /api/register
/// Register a new user account
pub async fn register(
    State(state): State<AppState>,
    Json(payload): Json<RegisterRequest>,
) -> Result<Json<ApiResponse<AuthResponse>>, ApiError> {
    // Validate input
    payload.validate().map_err(|e| {
        let mut errors = HashMap::new();
        for error in e.field_errors() {
            errors.insert(
                error.0.to_string(),
                error.1[0].message.clone().unwrap_or_default().to_string(),
            );
        }
        ApiError::with_errors(
            StatusCode::BAD_REQUEST,
            "Please correct the errors below",
            errors,
        )
    })?;

    // Check passwords match
    if payload.password != payload.confirm_password {
        let mut errors = HashMap::new();
        errors.insert("confirm_password".to_string(), "Passwords do not match".to_string());
        return Err(ApiError::with_errors(
            StatusCode::BAD_REQUEST,
            "Please correct the errors below",
            errors,
        ));
    }

    // Check if username already exists
    if state.db.username_exists(&payload.username).await
        .map_err(|e| ApiError::new(StatusCode::INTERNAL_SERVER_ERROR, &format!("Database error: {}", e)))? {
        let mut errors = HashMap::new();
        errors.insert("username".to_string(), "Username already exists".to_string());
        return Err(ApiError::with_errors(
            StatusCode::BAD_REQUEST,
            "Please correct the errors below",
            errors,
        ));
    }

    // Check if email already exists
    if state.db.email_exists(&payload.email).await
        .map_err(|e| ApiError::new(StatusCode::INTERNAL_SERVER_ERROR, &format!("Database error: {}", e)))? {
        let mut errors = HashMap::new();
        errors.insert("email".to_string(), "Email already exists".to_string());
        return Err(ApiError::with_errors(
            StatusCode::BAD_REQUEST,
            "Please correct the errors below",
            errors,
        ));
    }

    // Hash password
    let password_hash = state.password_service.hash_password(&payload.password)
        .map_err(|e| ApiError::new(StatusCode::INTERNAL_SERVER_ERROR, &format!("Failed to hash password: {}", e)))?;

    // Create user in database
    let user = state.db.create_user(&payload.username, &payload.email, &password_hash).await
        .map_err(|e| ApiError::new(StatusCode::INTERNAL_SERVER_ERROR, &format!("Failed to create user: {}", e)))?;

    // Generate JWT token
    let token = state.jwt_service.generate_token(&user.id, &user.username, user.is_admin)
        .map_err(|e| ApiError::new(StatusCode::INTERNAL_SERVER_ERROR, &format!("Failed to generate token: {}", e)))?;

    Ok(Json(ApiResponse::success(
        "Registration successful",
        AuthResponse { token, is_admin: user.is_admin },
    )))
}

/// POST /api/login
/// Authenticate user and return JWT token
pub async fn login(
    State(state): State<AppState>,
    Json(payload): Json<LoginRequest>,
) -> Result<Json<ApiResponse<AuthResponse>>, ApiError> {
    // Get user from database
    let user = state.db.get_user_by_username(&payload.username).await
        .map_err(|_| ApiError::new(StatusCode::UNAUTHORIZED, "Invalid username or password"))?;

    // Verify password
    let is_valid = state.password_service.verify_password(&payload.password, &user.password_hash)
        .map_err(|e| ApiError::new(StatusCode::INTERNAL_SERVER_ERROR, &format!("Password verification failed: {}", e)))?;

    if !is_valid {
        return Err(ApiError::new(StatusCode::UNAUTHORIZED, "Invalid username or password"));
    }

    // Generate JWT token
    let token = state.jwt_service.generate_token(&user.id, &user.username, user.is_admin)
        .map_err(|e| ApiError::new(StatusCode::INTERNAL_SERVER_ERROR, &format!("Failed to generate token: {}", e)))?;

    Ok(Json(ApiResponse::success(
        "Login successful",
        AuthResponse { token, is_admin: user.is_admin },
    )))
}

/// POST /api/logout
/// Invalidate the current JWT token (requires authentication)
pub async fn logout(
    Extension(_claims): Extension<Claims>,
) -> ApiResultNoData {
    // Note: With JWT, logout is typically handled client-side by removing the token
    // For server-side invalidation, you would need to implement a token blacklist
    // This endpoint confirms the client has authenticated before clearing their token
    
    Ok(Json(ApiResponse::no_data("Logged out successfully")))
}

/// POST /api/refresh
/// Refresh JWT token (requires authentication)
pub async fn refresh_token(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
) -> Result<Json<ApiResponse<AuthResponse>>, ApiError> {
    // Generate new token with same claims but renewed expiry
    let new_token = state.jwt_service.generate_token(&claims.sub, &claims.username, claims.is_admin)
        .map_err(|e| ApiError::new(StatusCode::INTERNAL_SERVER_ERROR, &format!("Failed to generate token: {}", e)))?;

    Ok(Json(ApiResponse::success(
        "Token refreshed",
        AuthResponse { token: new_token, is_admin: claims.is_admin },
    )))
}
