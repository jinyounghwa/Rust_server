/// Password Hashing and Verification
///
/// Handles password hashing with bcrypt and password strength validation.

use bcrypt::{hash, verify, DEFAULT_COST};

use crate::error::{AppError, ValidationError};

const MIN_PASSWORD_LENGTH: usize = 8;
const MAX_PASSWORD_LENGTH: usize = 128;

/// Hash a password using bcrypt
///
/// # Arguments
/// * `password` - Plain text password to hash
///
/// # Errors
/// Returns error if:
/// - Password fails validation (too short, weak, etc.)
/// - Bcrypt hashing fails
pub fn hash_password(password: &str) -> Result<String, AppError> {
    validate_password_strength(password)?;

    hash(password, DEFAULT_COST)
        .map_err(|e| AppError::Internal(format!("Password hashing failed: {}", e)))
}

/// Verify a password against its hash
///
/// # Arguments
/// * `password` - Plain text password to verify
/// * `hash` - Bcrypt hash to verify against
///
/// # Errors
/// Returns error if verification fails
pub fn verify_password(password: &str, hash: &str) -> Result<bool, AppError> {
    verify(password, hash)
        .map_err(|e| AppError::Internal(format!("Password verification failed: {}", e)))
}

/// Validate password strength requirements
///
/// Requirements:
/// - Minimum 8 characters
/// - Maximum 128 characters
/// - At least one digit
/// - At least one lowercase letter
/// - At least one uppercase letter
fn validate_password_strength(password: &str) -> Result<(), AppError> {
    // Check minimum length
    if password.len() < MIN_PASSWORD_LENGTH {
        return Err(AppError::Validation(ValidationError::TooShort(
            "password".to_string(),
            MIN_PASSWORD_LENGTH,
        )));
    }

    // Check maximum length (bcrypt limitation and DoS prevention)
    if password.len() > MAX_PASSWORD_LENGTH {
        return Err(AppError::Validation(ValidationError::TooLong(
            "password".to_string(),
            MAX_PASSWORD_LENGTH,
        )));
    }

    // Check for at least one digit, one lowercase, and one uppercase
    let has_digit = password.chars().any(|c| c.is_numeric());
    let has_lowercase = password.chars().any(|c| c.is_lowercase());
    let has_uppercase = password.chars().any(|c| c.is_uppercase());

    if !has_digit || !has_lowercase || !has_uppercase {
        return Err(AppError::Validation(ValidationError::InvalidFormat(
            "password must contain at least one digit, one lowercase letter, and one uppercase letter"
                .to_string(),
        )));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hash_password() {
        let password = "ValidPassword123";
        let hash = hash_password(password).expect("Failed to hash password");

        // Hash should not be the same as password
        assert_ne!(password, hash);
        // Hash should start with bcrypt identifier
        assert!(hash.starts_with("$2"));
    }

    #[test]
    fn test_verify_password() {
        let password = "ValidPassword123";
        let hash = hash_password(password).expect("Failed to hash password");

        let is_valid = verify_password(password, &hash).expect("Failed to verify password");
        assert!(is_valid);
    }

    #[test]
    fn test_verify_wrong_password() {
        let password = "ValidPassword123";
        let hash = hash_password(password).expect("Failed to hash password");

        let is_valid = verify_password("WrongPassword123", &hash).expect("Failed to verify password");
        assert!(!is_valid);
    }

    #[test]
    fn test_too_short_password() {
        let result = hash_password("Short1");
        assert!(result.is_err());
    }

    #[test]
    fn test_too_long_password() {
        let long_password = "a".repeat(MAX_PASSWORD_LENGTH + 1) + "A1";
        let result = hash_password(&long_password);
        assert!(result.is_err());
    }

    #[test]
    fn test_no_digits() {
        let result = hash_password("NoDigitsPassword");
        assert!(result.is_err());
    }

    #[test]
    fn test_no_lowercase() {
        let result = hash_password("NOLOWERCASE1");
        assert!(result.is_err());
    }

    #[test]
    fn test_no_uppercase() {
        let result = hash_password("nouppercase1");
        assert!(result.is_err());
    }

    #[test]
    fn test_valid_password() {
        let result = hash_password("ValidPassword123");
        assert!(result.is_ok());
    }
}
