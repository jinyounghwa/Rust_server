/// JWT Token Generation and Validation
///
/// Handles creation and validation of JWT tokens for authentication.

use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};
use uuid::Uuid;

use crate::auth::claims::Claims;
use crate::configuration::JwtSettings;
use crate::error::AppError;

/// Generate a new access token for a user
///
/// # Arguments
/// * `user_id` - User's UUID
/// * `email` - User's email address
/// * `config` - JWT configuration settings
///
/// # Errors
/// Returns error if token generation fails
pub fn generate_access_token(
    user_id: &Uuid,
    email: &str,
    config: &JwtSettings,
) -> Result<String, AppError> {
    let claims = Claims::new(
        *user_id,
        email.to_string(),
        config.access_token_expiry,
        config.issuer.clone(),
    );

    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(config.secret.as_bytes()),
    )
    .map_err(|e| AppError::Internal(format!("Token generation failed: {}", e)))
}

/// Validate and extract claims from an access token
///
/// # Arguments
/// * `token` - JWT token string
/// * `config` - JWT configuration settings
///
/// # Errors
/// Returns error if token is invalid, expired, or tampered with
pub fn validate_access_token(token: &str, config: &JwtSettings) -> Result<Claims, AppError> {
    let mut validation = Validation::new(Algorithm::HS256);
    // Verify issuer matches configuration
    validation.set_issuer(&[&config.issuer]);

    decode::<Claims>(
        token,
        &DecodingKey::from_secret(config.secret.as_bytes()),
        &validation,
    )
    .map(|data| data.claims)
    .map_err(|e| {
        tracing::warn!("JWT validation error: {}", e);
        AppError::Internal("Invalid or expired token".to_string())
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn get_test_config() -> JwtSettings {
        JwtSettings {
            secret: "test-secret-key-at-least-32-characters-long".to_string(),
            access_token_expiry: 3600,
            refresh_token_expiry: 604800,
            issuer: "test".to_string(),
        }
    }

    #[test]
    fn test_generate_and_validate_token() {
        let config = get_test_config();
        let user_id = Uuid::new_v4();
        let email = "test@example.com";

        let token = generate_access_token(&user_id, email, &config).expect("Failed to generate token");
        let claims = validate_access_token(&token, &config).expect("Failed to validate token");

        assert_eq!(claims.sub, user_id.to_string());
        assert_eq!(claims.email, email);
        assert_eq!(claims.iss, "test");
    }

    #[test]
    fn test_invalid_token() {
        let config = get_test_config();
        let result = validate_access_token("invalid.token.here", &config);

        assert!(result.is_err());
    }

    #[test]
    fn test_tampered_token() {
        let config = get_test_config();
        let user_id = Uuid::new_v4();

        let token = generate_access_token(&user_id, "test@example.com", &config)
            .expect("Failed to generate token");

        // Tamper with token
        let tampered = format!("{}X", token);
        let result = validate_access_token(&tampered, &config);

        assert!(result.is_err());
    }

    #[test]
    fn test_wrong_issuer() {
        let mut config = get_test_config();
        let user_id = Uuid::new_v4();

        let token = generate_access_token(&user_id, "test@example.com", &config)
            .expect("Failed to generate token");

        // Change issuer in validation config
        config.issuer = "wrong-issuer".to_string();
        let result = validate_access_token(&token, &config);

        assert!(result.is_err());
    }
}
