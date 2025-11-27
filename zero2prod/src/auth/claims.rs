/// JWT Claims structure
///
/// Represents the payload of a JWT token containing user information
/// and standard JWT claims (RFC 7519).

use serde::{Deserialize, Serialize};
use uuid::Uuid;
use crate::error::AppError;

/// JWT Claims for access tokens
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Claims {
    /// Subject (user ID as UUID string)
    pub sub: String,
    /// User email
    pub email: String,
    /// Expiration time (Unix timestamp)
    pub exp: i64,
    /// Issued at (Unix timestamp)
    pub iat: i64,
    /// Issuer
    pub iss: String,
}

impl Claims {
    /// Create new claims with user information
    ///
    /// # Arguments
    /// * `user_id` - User's UUID
    /// * `email` - User's email address
    /// * `expiry_seconds` - Token expiration in seconds from now
    /// * `issuer` - Issuer identifier
    pub fn new(
        user_id: Uuid,
        email: String,
        expiry_seconds: i64,
        issuer: String,
    ) -> Self {
        let now = chrono::Utc::now().timestamp();
        Self {
            sub: user_id.to_string(),
            email,
            exp: now + expiry_seconds,
            iat: now,
            iss: issuer,
        }
    }

    /// Extract user ID from claims
    ///
    /// # Errors
    /// Returns error if user ID is not a valid UUID
    pub fn user_id(&self) -> Result<Uuid, AppError> {
        Uuid::parse_str(&self.sub)
            .map_err(|_| AppError::Internal("Invalid user ID in token".to_string()))
    }

    /// Check if token has expired
    pub fn is_expired(&self) -> bool {
        let now = chrono::Utc::now().timestamp();
        self.exp < now
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_claims_creation() {
        let user_id = Uuid::new_v4();
        let email = "test@example.com".to_string();
        let claims = Claims::new(user_id, email.clone(), 3600, "test".to_string());

        assert_eq!(claims.sub, user_id.to_string());
        assert_eq!(claims.email, email);
        assert_eq!(claims.iss, "test");
        assert!(!claims.is_expired());
    }

    #[test]
    fn test_user_id_extraction() {
        let user_id = Uuid::new_v4();
        let claims = Claims::new(user_id, "test@example.com".to_string(), 3600, "test".to_string());

        assert_eq!(claims.user_id().unwrap(), user_id);
    }

    #[test]
    fn test_invalid_user_id() {
        let mut claims = Claims::new(
            Uuid::new_v4(),
            "test@example.com".to_string(),
            3600,
            "test".to_string(),
        );
        claims.sub = "invalid-uuid".to_string();

        assert!(claims.user_id().is_err());
    }
}
