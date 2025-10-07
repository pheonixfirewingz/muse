use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;

#[derive(Clone)]
pub struct JwtService {
    encoding_key: EncodingKey,
    decoding_key: DecodingKey,
    expiration_hours: i64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Claims {
    pub sub: String,      // Subject (user ID)
    pub username: String, // Username
    pub is_admin: bool,   // Admin status
    pub exp: i64,         // Expiration time
    pub iat: i64,         // Issued at
}

impl JwtService {
    pub fn new(secret: &str, expiration_hours: i64) -> Self {
        Self {
            encoding_key: EncodingKey::from_secret(secret.as_bytes()),
            decoding_key: DecodingKey::from_secret(secret.as_bytes()),
            expiration_hours,
        }
    }

    pub fn generate_token(&self, user_id: &str, username: &str, is_admin: bool) -> Result<String, jsonwebtoken::errors::Error> {
        let now = OffsetDateTime::now_utc().unix_timestamp();
        let expiration = now + (self.expiration_hours * 3600);

        let claims = Claims {
            sub: user_id.to_string(),
            username: username.to_string(),
            is_admin,
            exp: expiration,
            iat: now,
        };

        encode(&Header::default(), &claims, &self.encoding_key)
    }

    pub fn verify_token(&self, token: &str) -> Result<Claims, jsonwebtoken::errors::Error> {
        let token_data = decode::<Claims>(
            token,
            &self.decoding_key,
            &Validation::default(),
        )?;

        Ok(token_data.claims)
    }

    #[allow(dead_code)]
    pub fn refresh_token(&self, old_token: &str) -> Result<String, jsonwebtoken::errors::Error> {
        let claims = self.verify_token(old_token)?;
        self.generate_token(&claims.sub, &claims.username, claims.is_admin)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_jwt_generation_and_verification() {
        let jwt_service = JwtService::new("test_secret_key_for_testing", 24);
        let token = jwt_service.generate_token("user123", "testuser", false).unwrap();
        
        let claims = jwt_service.verify_token(&token).unwrap();
        assert_eq!(claims.sub, "user123");
        assert_eq!(claims.username, "testuser");
        assert_eq!(claims.is_admin, false);
    }

    #[test]
    fn test_jwt_refresh() {
        let jwt_service = JwtService::new("test_secret_key_for_testing", 24);
        let original_token = jwt_service.generate_token("user123", "testuser", true).unwrap();
        
        let new_token = jwt_service.refresh_token(&original_token).unwrap();
        let claims = jwt_service.verify_token(&new_token).unwrap();
        
        assert_eq!(claims.sub, "user123");
        assert_eq!(claims.username, "testuser");
        assert_eq!(claims.is_admin, true);
    }
}
