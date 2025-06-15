use regex::Regex;
use rustrict::CensorStr;
use serde::Deserialize;
use std::sync::LazyLock;
use validator::{Validate, ValidationError};

static RESERVED_NAMES:&[&str] = &[
    "admin", "administrator", "root", "system",
    "moderator", "support", "help", "info",
    "webmaster", "security", "staff", "team",
    "anonymous", "undefined", "null", "test"
];

static USERNAME_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^[a-zA-Z0-9_]+$").unwrap()
});

#[derive(Debug, Deserialize, Validate)]
pub struct RegisterForm {
    #[validate(
        length(min = 3, max = 20, message = "Username must be between 3 and 20 characters"),
        regex(path = *USERNAME_REGEX, message = "Username can only contain letters, numbers, and underscores"),
        custom(function = "validate_no_profanity")
    )]
    pub username: String,

    #[validate(email(message = "Please enter a valid email address"))]
    pub email: String,

    #[validate(
        length(min = 8, max = 32, message = "Password must be between 8 and 32 characters"),
        custom(function = "validate_password_strength")
    )]
    pub password: String,

    pub confirm_password: String,
}

impl RegisterForm {
    pub(crate) fn validate_password_match(&self) -> Result<(), ValidationError> {
        if self.password != self.confirm_password {
            return Err(ValidationError::new("password_mismatch"));
        }
        Ok(())
    }
}

fn validate_password_strength(password: &str) -> Result<(), ValidationError> {
    let has_lowercase = password.chars().any(|c| c.is_ascii_lowercase());
    let has_uppercase = password.chars().any(|c| c.is_ascii_uppercase());
    let has_digit = password.chars().any(|c| c.is_ascii_digit());
    let has_special = password.chars().any(|c| !c.is_alphanumeric());

    if !has_lowercase || !has_uppercase || !has_digit || !has_special {
        return Err(ValidationError::new("password_complexity"));
    }
    Ok(())
}

fn validate_no_profanity(username: &str) -> Result<(), ValidationError> {
    let username_lower = username.to_lowercase();
    if RESERVED_NAMES.iter().any(|&word| username_lower.contains(word)) {
        return Err(ValidationError::new("reserved"));
    }
    if username_lower.is_inappropriate() {
        return Err(ValidationError::new("inappropriate"));
    }
    Ok(())
}