use crate::db::schema::session::create_session;
use crate::db::schema::user::User;
use crate::login::login_form::LoginForm;
use crate::login::register_form::RegisterForm;
use crate::{db, AppState};
use axum::extract::State;
use axum::http::{HeaderMap, StatusCode};
use axum::response::{Html, IntoResponse};
use axum::routing::{get, post};
use axum::{Form, Json, Router};
use bcrypt::hash;
use serde::Serialize;
use std::collections::HashMap;
use std::sync::Arc;
use tower_cookies::{Cookie, Cookies};
use validator::Validate;
use tracing::{error, info, warn, debug};
const BCRYPT_COST: u32 = 14;

#[derive(Serialize)]
pub struct SuccessResponse {
    pub success: bool,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub redirect: Option<String>,
}

#[derive(Serialize)]
pub struct ErrorResponse {
    pub success: bool,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub errors: Option<HashMap<String, String>>,
}

pub async fn login_handler() -> impl IntoResponse {
    debug!("Serving login page");
    Html(include_str!("../../statics/login.html"))
}

pub async fn login_submit(
    State(state): State<Arc<AppState>>,
    cookies: Cookies,
    headers: HeaderMap,
    Form(mut form): Form<LoginForm>,
) -> Result<Json<SuccessResponse>, Json<ErrorResponse>> {
    info!("Processing login attempt for user: {}", form.username);
    
    let content_type = headers
        .get("content-type")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");
    if !content_type.contains("application/x-www-form-urlencoded") {
        warn!("Invalid content type received: {}", content_type);
        form.password.clear();
        return Err(Json(ErrorResponse {
            success: false,
            message: "Invalid Content-Type. Expected application/x-www-form-urlencoded".to_string(),
            errors: None,
        }));
    }
    
    let db = &state.db;
    if !db::schema::user::is_valid_user(&db, &None, &Some(&form.username), &form.password).await {
        warn!("Invalid login attempt for user: {}", form.username);
        form.password.clear();
        return Err(Json(ErrorResponse {
            success: false,
            message: "Invalid username or password".to_string(),
            errors: None,
        }));
    }
    info!("Valid credentials for user: {}", form.username);
    form.password.clear();

    let user_id = match db::schema::user::get_user_id_by_username(&db, &form.username).await {
        Some(id) => id,
        None => {
            error!("Failed to get user ID for authenticated user: {}", form.username);
            return Err(Json(ErrorResponse {
                success: false,
                message: "Failed to create session".to_string(),
                errors: None,
            }));
        }
    };

    debug!("Creating session for user_id: {}", user_id);
    let session_id = create_session(&db, &user_id).await;

    let cookie = Cookie::build(("session_id", session_id))
        .path("/")
        .secure(true)
        .http_only(true)
        .same_site(tower_cookies::cookie::SameSite::Strict)
        .max_age(time::Duration::hours(24))
        .into();

    info!("Login successful for user: {}", form.username);
    cookies.add(cookie);
    
    Ok(Json(SuccessResponse {
        success: true,
        message: "Login successful".to_string(),
        redirect: Some("/app".to_string()),
    }))
}

pub async fn register_handler() -> impl IntoResponse {
    debug!("Serving registration page");
    Html(include_str!("../../statics/register.html"))
}

pub async fn register_submit(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Form(mut form): Form<RegisterForm>,
) -> Result<Json<SuccessResponse>, (StatusCode, Json<ErrorResponse>)> {
    info!("Processing registration for username: {}", form.username);
    
    let content_type = headers
        .get("content-type")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");

    if !content_type.contains("application/x-www-form-urlencoded") {
        warn!("Invalid content type for registration: {}", content_type);
        return Err((
            StatusCode::UNSUPPORTED_MEDIA_TYPE,
            Json(ErrorResponse {
                success: false,
                message: "Invalid Content-Type. Expected application/x-www-form-urlencoded".to_string(),
                errors: None,
            }),
        ));
    }

    form.username = form.username.trim().to_string();
    form.email = form.email.trim().to_lowercase();
    debug!("Validating registration form for user: {}", form.username);

    let mut field_errors = HashMap::new();

    if let Err(errors) = form.validate() {
        debug!("Validation errors in registration form");
        for (field, field_errors_vec) in errors.field_errors() {
            if let Some(error) = field_errors_vec.first() {
                let message = match error.code.as_ref() {
                    "length" => error.message.as_ref().map(|m| m.to_string())
                        .unwrap_or_else(|| "Invalid length".to_string()),
                    "email" => "Please enter a valid email address".to_string(),
                    "regex" => "Username can only contain letters, numbers, and underscores".to_string(),
                    "profanity" => "Username contains restricted words or inappropriate content".to_string(),
                    "password_strength" => "Password must contain at least one number and one special character (!@#$%^&*)".to_string(),
                    "reserved" => "Username contains a reserved word".to_string(),
                    "inappropriate" => "Username contains inappropriate content".to_string(),
                    _ => "Invalid input".to_string(),
                };
                field_errors.insert(field.to_string(), message.clone());
                warn!("Registration validation error - Field: {}, Error: {}", field, message);
            }
        }
    }

    if let Err(_) = form.validate_password_match() {
        debug!("Password mismatch during registration for user: {}", form.username);
        field_errors.insert("confirm_password".to_string(), "Passwords do not match".to_string());
        warn!("Registration validation error - Passwords do not match");
    }

    if !field_errors.is_empty() {
        form.password.clear();
        form.confirm_password.clear();

        return Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                success: false,
                message: "Please correct the errors below".to_string(),
                errors: Some(field_errors),
            })
        ));
    }
    
    let password_hash = hash(&form.password, BCRYPT_COST).unwrap();
    form.password.clear();
    form.confirm_password.clear();
    
    let user: User = User::new(form.username,form.email,password_hash);
    let db = &state.db;
    if db::schema::user::check_if_username_is_taken(&db,user.get_name()).await {
        warn!("Registration failed - Username already taken: {}", user.get_name());
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                success: false,
                message: "Username is already taken".to_string(),
                errors: None,
            })
        ));
    }
    
    if db::schema::user::check_if_email_is_taken(&db,user.get_email()).await {
        warn!("Registration failed - Email already taken: {}", user.get_email());
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                success: false,
                message: "Email is already taken".to_string(),
                errors: None,
            })
        ));   
    }
    
    db::schema::user::insert_user(&db,&user).await;
    info!("Registration successful for user: {}",&user.get_name());
    Ok(Json(SuccessResponse {
        success: true,
        message: "Registration successful".to_string(),
        redirect: Some("/login".to_string()),
    }))
}

pub async fn logout_handler() -> impl IntoResponse {
    Html("<h1>Login Page</h1>")
}

pub async fn profile_handler() -> impl IntoResponse {
    Html("<h1>Login Page</h1>")
}

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/login", get(login_handler))
        .route("/login/submit", post(login_submit))
        .route("/register", get(register_handler))
        .route("/register/submit", post(register_submit))
        .route("/logout", get(logout_handler))
        .route("/profile", get(profile_handler))
}