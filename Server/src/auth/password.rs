use bcrypt::{hash, verify, DEFAULT_COST};

#[derive(Debug, Clone)]
pub struct PasswordService {
    cost: u32,
}

impl PasswordService {
    pub fn new() -> Self {
        Self {
            cost: DEFAULT_COST,
        }
    }

    pub fn hash_password(&self, password: &str) -> Result<String, bcrypt::BcryptError> {
        hash(password, self.cost)
    }

    pub fn verify_password(&self, password: &str, hash: &str) -> Result<bool, bcrypt::BcryptError> {
        verify(password, hash)
    }
}

impl Default for PasswordService {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_password_hashing_and_verification() {
        let password_service = PasswordService::new();
        let password = "SecurePassword123!";
        
        let hash = password_service.hash_password(password).unwrap();
        assert_ne!(hash, password);
        
        let is_valid = password_service.verify_password(password, &hash).unwrap();
        assert!(is_valid);
        
        let is_invalid = password_service.verify_password("WrongPassword", &hash).unwrap();
        assert!(!is_invalid);
    }
}
