use axum::{extract::{Json, State, Extension}, http::StatusCode};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use crate::api::response::{ApiResponse, ApiResult, ApiResultNoData, ApiError};
use crate::api::auth::AppState;
use crate::auth::jwt::Claims;

#[derive(Debug, Deserialize)]
pub struct UpdateUserRequest {
    pub username: Option<String>,
    pub email: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ChangePasswordRequest {
    pub old_password: String,
    pub new_password: String,
}

#[derive(Debug, Deserialize)]
pub struct ResetPasswordRequest {
    pub email: String,
}

#[derive(Debug, Deserialize)]
pub struct DeleteAccountRequest {
    pub password: String,
}

#[derive(Debug, Serialize)]
pub struct UserInfo {
    pub username: String,
    pub email: String,
    pub created_at: String,
    pub is_admin: bool,
}

pub async fn get_user_info(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
) -> ApiResult<UserInfo> {
    // Get user from database using ID from claims
    let user = state.db.get_user_by_id(&claims.sub).await
        .map_err(|e| ApiError::new(StatusCode::INTERNAL_SERVER_ERROR, &format!("Failed to get user: {}", e)))?;
    
    let user_info = UserInfo {
        username: user.username,
        email: user.email,
        created_at: user.created_at.format(&time::format_description::well_known::Rfc3339)
            .unwrap_or_else(|_| "2025-01-01T00:00:00Z".to_string()),
        is_admin: user.is_admin,
    };
    
    Ok(Json(ApiResponse::success("User info retrieved successfully", user_info)))
}

pub async fn update_user_info(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Json(payload): Json<UpdateUserRequest>,
) -> ApiResultNoData {
    // Check if at least one field is being updated
    if payload.username.is_none() && payload.email.is_none() {
        return Err(ApiError::new(StatusCode::BAD_REQUEST, "No fields to update"));
    }
    
    // Update username if provided
    if let Some(new_username) = &payload.username {
        // Validate username length
        if new_username.len() < 3 || new_username.len() > 20 {
            let mut errors = HashMap::new();
            errors.insert("username".to_string(), "Username must be between 3 and 20 characters".to_string());
            return Err(ApiError::with_errors(
                StatusCode::BAD_REQUEST,
                "Please correct the errors below",
                errors,
            ));
        }
        
        // Check if username is already taken (but not by this user)
        if new_username != &claims.username {
            if state.db.username_exists(new_username).await
                .map_err(|e| ApiError::new(StatusCode::INTERNAL_SERVER_ERROR, &format!("Database error: {}", e)))? {
                let mut errors = HashMap::new();
                errors.insert("username".to_string(), "Username already exists".to_string());
                return Err(ApiError::with_errors(
                    StatusCode::BAD_REQUEST,
                    "Please correct the errors below",
                    errors,
                ));
            }
        }
        
        // Update username in database
        state.db.update_username(&claims.sub, new_username).await
            .map_err(|e| ApiError::new(StatusCode::INTERNAL_SERVER_ERROR, &format!("Failed to update username: {}", e)))?;
    }
    
    // Update email if provided
    if let Some(new_email) = &payload.email {
        // Basic email validation
        if !new_email.contains('@') || !new_email.contains('.') {
            let mut errors = HashMap::new();
            errors.insert("email".to_string(), "Invalid email format".to_string());
            return Err(ApiError::with_errors(
                StatusCode::BAD_REQUEST,
                "Please correct the errors below",
                errors,
            ));
        }
        
        // Check if email is already taken
        if state.db.email_exists(new_email).await
            .map_err(|e| ApiError::new(StatusCode::INTERNAL_SERVER_ERROR, &format!("Database error: {}", e)))? {
            // Get user's current email to see if it's the same
            let user = state.db.get_user_by_id(&claims.sub).await
                .map_err(|e| ApiError::new(StatusCode::INTERNAL_SERVER_ERROR, &format!("Failed to get user: {}", e)))?;
            
            if new_email != &user.email {
                let mut errors = HashMap::new();
                errors.insert("email".to_string(), "Email already exists".to_string());
                return Err(ApiError::with_errors(
                    StatusCode::BAD_REQUEST,
                    "Please correct the errors below",
                    errors,
                ));
            }
        }
        
        // Update email in database
        state.db.update_user_email(&claims.username, new_email).await
            .map_err(|e| ApiError::new(StatusCode::INTERNAL_SERVER_ERROR, &format!("Failed to update email: {}", e)))?;
    }
    
    Ok(Json(ApiResponse::no_data("User information updated successfully")))
}

pub async fn change_password(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Json(payload): Json<ChangePasswordRequest>,
) -> ApiResultNoData {
    // Validate new password length
    if payload.new_password.len() < 8 {
        let mut errors = HashMap::new();
        errors.insert("new_password".to_string(), "Password must be at least 8 characters".to_string());
        return Err(ApiError::with_errors(
            StatusCode::BAD_REQUEST,
            "Please correct the errors below",
            errors,
        ));
    }
    
    // Get user from database
    let user = state.db.get_user_by_id(&claims.sub).await
        .map_err(|e| ApiError::new(StatusCode::INTERNAL_SERVER_ERROR, &format!("Failed to get user: {}", e)))?;
    
    // Verify old password
    let is_valid = state.password_service.verify_password(&payload.old_password, &user.password_hash)
        .map_err(|e| ApiError::new(StatusCode::INTERNAL_SERVER_ERROR, &format!("Password verification failed: {}", e)))?;
    
    if !is_valid {
        let mut errors = HashMap::new();
        errors.insert("old_password".to_string(), "Incorrect password".to_string());
        return Err(ApiError::with_errors(
            StatusCode::BAD_REQUEST,
            "Please correct the errors below",
            errors,
        ));
    }
    
    // Hash new password
    let new_password_hash = state.password_service.hash_password(&payload.new_password)
        .map_err(|e| ApiError::new(StatusCode::INTERNAL_SERVER_ERROR, &format!("Failed to hash password: {}", e)))?;
    
    // Update password in database
    state.db.update_user_password(&claims.sub, &new_password_hash).await
        .map_err(|e| ApiError::new(StatusCode::INTERNAL_SERVER_ERROR, &format!("Failed to update password: {}", e)))?;
    
    Ok(Json(ApiResponse::no_data("Password changed successfully")))
}

pub async fn reset_password(
    State(state): State<AppState>,
    Json(payload): Json<ResetPasswordRequest>,
) -> ApiResultNoData {
    // Verify that the email exists in the database
    let _user = state.db.get_user_by_email(&payload.email).await
        .map_err(|_| ApiError::new(StatusCode::NOT_FOUND, "Email not found"))?;
    
    // TODO: Implement actual email sending with reset token
    // For now, just return success message
    // In production, you would:
    // 1. Generate a secure reset token
    // 2. Store it in the database with expiration
    // 3. Send email with reset link containing the token
    
    Ok(Json(ApiResponse::no_data("Password reset email sent")))
}

pub async fn delete_account(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Json(payload): Json<DeleteAccountRequest>,
) -> ApiResultNoData {
    // Get user from database
    let user = state.db.get_user_by_id(&claims.sub).await
        .map_err(|e| ApiError::new(StatusCode::INTERNAL_SERVER_ERROR, &format!("Failed to get user: {}", e)))?;
    
    // Verify password before allowing deletion
    let is_valid = state.password_service.verify_password(&payload.password, &user.password_hash)
        .map_err(|e| ApiError::new(StatusCode::INTERNAL_SERVER_ERROR, &format!("Password verification failed: {}", e)))?;
    
    if !is_valid {
        let mut errors = HashMap::new();
        errors.insert("password".to_string(), "Incorrect password".to_string());
        return Err(ApiError::with_errors(
            StatusCode::BAD_REQUEST,
            "Please correct the errors below",
            errors,
        ));
    }
    
    // Delete user from database
    state.db.delete_user_by_id(&claims.sub).await
        .map_err(|e| ApiError::new(StatusCode::INTERNAL_SERVER_ERROR, &format!("Failed to delete account: {}", e)))?;
    
    Ok(Json(ApiResponse::no_data("Account deleted successfully")))
}
