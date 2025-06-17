use crate::db::session::create_session;
use crate::db::user::User;
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
use tower_cookies::cookie::time::Duration;
use validator::Validate;
use tracing::{error, info, warn, debug};
use tower_governor::GovernorLayer;
use tower_governor::governor::GovernorConfigBuilder;

pub(crate) const BCRYPT_COST: u32 = 14;

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

pub fn check_header(headers: &HeaderMap) -> bool {
    let content_type = headers.get("content-type")
        .and_then(|v| v.to_str().ok()).unwrap_or("").trim();

    !content_type.contains("application/x-www-form-urlencoded")
}

pub async fn login_submit(
    State(state): State<Arc<AppState>>,
    cookies: Cookies,
    headers: HeaderMap,
    Form(mut form): Form<LoginForm>,
) -> Result<Json<SuccessResponse>, Json<ErrorResponse>> {
    if check_header(&headers) {
        let content_type = headers.get("content-type")
            .and_then(|v| v.to_str().ok()).unwrap_or("").trim();
        warn!("Invalid content type received: {}", content_type);
        form.password.clear();
        return Err(Json(ErrorResponse {
            success: false,
            message: "Invalid Content-Type. Expected application/x-www-form-urlencoded".to_string(),
            errors: None,
        }));
    }

    info!("Processing login attempt for user: {}", form.username);
    let db = &state.db;
    let result = db::user::is_valid_user(&db, &None, &Some(&form.username), &form.password).await;
    form.password.clear();
    if result.is_err() {
        warn!("Failed to validate user credentials: {:?}",form);
        return Err(Json(ErrorResponse {
            success: false,
            message: "Internal Server Error".to_string(),
            errors: None,
        }));
    } else {
        let result = result.unwrap();
        if !result {
            return Err(Json(ErrorResponse {
                success: false,
                message: "Invalid credentials".to_string(),
                errors: None,
            }));
        }
    }
    info!("Valid credentials for user: {}", form.username);
    let user_id = match db::user::get_user_uuid_by_username(&db, &form.username).await {
        Ok(id) => id,
        Err(e) => {
            error!("Failed to get user ID for authenticated user: {} -> {:?}", form.username, e);
            return Err(Json(ErrorResponse {
                success: false,
                message: "Failed to find user with requested username".to_string(),
                errors: None,
            }));
        }
    };
    debug!("Creating session for user: {}", form.username);
    let session_id = create_session(&db, &user_id).await.unwrap().to_string();

    let cookie = Cookie::build(("session_id", session_id))
        .path("/").secure(true).http_only(true)
        .same_site(tower_cookies::cookie::SameSite::Strict)
        .max_age(Duration::hours(24)).into();
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
    if check_header(&headers) {
        let content_type = headers.get("content-type")
            .and_then(|v| v.to_str().ok()).unwrap_or("").trim();
        warn!("Invalid content type received: {}", content_type);
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                success: false,
                message: "Invalid Content-Type. Expected application/x-www-form-urlencoded".to_string(),
                errors: None,
            }),
        ));
    }

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
    info!("Processing registration for username: {}", form.username);
    form.username = form.username.trim().to_string();
    form.email = form.email.trim().to_lowercase();
    let mut password = form.password.clone();
    let password_hash = async {
        //https://docs.rs/bcrypt/0.17.0/bcrypt/fn.hash.html
        //Generates a password hash using the cost given. The salt is generated randomly using the OS randomness
        let result = hash(&password, BCRYPT_COST).unwrap();
        password.clear();
        result
    };
    form.password.clear();
    form.confirm_password.clear();
    let db = &state.db;
    let user: User = User::new(form.username.as_str(), form.email.as_str(), password_hash.await.to_string().as_str());
    if let Ok(db_result) = db::user::create_user_if_not_exists(&db,&user).await {
        if db_result {
            info!("Registration successful for user: {}", form.username);
            Ok(Json(SuccessResponse {
                success: true,
                message: "Registration successful".to_string(),
                redirect: Some("/login".to_string()),
            }))
        } else {
            warn!("the email or user is already registered with another account");
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    success: false,
                    message: "Failed to create session".to_string(),
                    errors: None,
                })
            ))
        }

    } else {
        warn!("Failed to insert user into database");
        Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                success: false,
                message: "Failed to create session".to_string(),
                errors: None,
            })
        ))
    }
}

pub async fn profile_handler() -> impl IntoResponse {
    Html("<h1>Login Page</h1>")
}

pub fn router() -> Router<Arc<AppState>> {
    let governor_conf = Arc::new(
        GovernorConfigBuilder::default()
            .per_second(2)
            .burst_size(5)
            .finish()
            .unwrap(),
    );
    
    let limited = Router::new()
        .route("/login/submit", post(login_submit))
        .route("/register/submit", post(register_submit))
        .layer(GovernorLayer {
            config: governor_conf,
        });

    Router::new()
        .route("/login", get(login_handler))
        .route("/register", get(register_handler))
        .route("/profile", get(profile_handler))
        .merge(limited)
}