/// Data Validation module - validates stored data in the database
/// Features:
/// 1. Data Integrity Checks: Validates data from database
/// 2. Email Validation: Ensures stored emails are valid
/// 3. Name Validation: Ensures stored names are valid
/// 4. Status Validation: Ensures subscription status is valid
/// 5. Data Consistency: Validates relationships between data

use crate::error::ValidationError;
use crate::validators::is_valid_email;

const VALID_STATUSES: &[&str] = &["pending", "confirmed"];
const MIN_NAME_LENGTH: usize = 1;
const MAX_NAME_LENGTH: usize = 256;

/// Validates a subscriber record from the database
pub fn validate_subscriber_data(
    id: &str,
    email: &str,
    name: &str,
    status: &str,
) -> Result<(), ValidationError> {
    // Validate UUID format (basic check)
    validate_uuid(id)?;

    // Validate email
    is_valid_email(email)?;

    // Validate name
    validate_stored_name(name)?;

    // Validate status
    validate_subscription_status(status)?;

    Ok(())
}

/// Validates UUID format
pub fn validate_uuid(id: &str) -> Result<(), ValidationError> {
    let trimmed = id.trim();

    if trimmed.is_empty() {
        return Err(ValidationError::EmptyField("id".to_string()));
    }

    // Basic UUID v4 format validation (8-4-4-4-12 hex characters)
    let uuid_pattern = regex::Regex::new(
        r"^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$"
    ).unwrap();

    if !uuid_pattern.is_match(trimmed) {
        return Err(ValidationError::InvalidFormat("id".to_string()));
    }

    Ok(())
}

/// Validates stored name from database
pub fn validate_stored_name(name: &str) -> Result<(), ValidationError> {
    let trimmed = name.trim();

    if trimmed.is_empty() {
        return Err(ValidationError::EmptyField("name".to_string()));
    }

    if trimmed.len() < MIN_NAME_LENGTH {
        return Err(ValidationError::TooShort("name".to_string(), MIN_NAME_LENGTH));
    }

    if trimmed.len() > MAX_NAME_LENGTH {
        return Err(ValidationError::TooLong("name".to_string(), MAX_NAME_LENGTH));
    }

    // Check for null bytes
    if trimmed.contains('\0') {
        return Err(ValidationError::SuspiciousContent("name".to_string()));
    }

    // Check for control characters
    if trimmed.chars().any(|c| c.is_control()) {
        return Err(ValidationError::SuspiciousContent("name".to_string()));
    }

    Ok(())
}

/// Validates subscription status
pub fn validate_subscription_status(status: &str) -> Result<(), ValidationError> {
    let trimmed = status.trim();

    if trimmed.is_empty() {
        return Err(ValidationError::EmptyField("status".to_string()));
    }

    if !VALID_STATUSES.contains(&trimmed) {
        return Err(ValidationError::SuspiciousContent(
            format!("Invalid status '{}'. Valid statuses are: {}", trimmed, VALID_STATUSES.join(", "))
        ));
    }

    Ok(())
}

/// Batch validates multiple subscribers
pub fn validate_subscribers_batch(
    subscribers: &[(String, String, String, String)],
) -> Result<(), (usize, ValidationError)> {
    for (idx, (id, email, name, status)) in subscribers.iter().enumerate() {
        if let Err(e) = validate_subscriber_data(id, email, name, status) {
            return Err((idx, e));
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_subscriber_data_valid() {
        let result = validate_subscriber_data(
            "550e8400-e29b-41d4-a716-446655440000",
            "user@example.com",
            "John Doe",
            "confirmed",
        );
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_subscriber_data_invalid_uuid() {
        let result = validate_subscriber_data(
            "invalid-uuid",
            "user@example.com",
            "John Doe",
            "confirmed",
        );
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_subscriber_data_invalid_email() {
        let result = validate_subscriber_data(
            "550e8400-e29b-41d4-a716-446655440000",
            "invalid-email",
            "John Doe",
            "confirmed",
        );
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_subscriber_data_invalid_status() {
        let result = validate_subscriber_data(
            "550e8400-e29b-41d4-a716-446655440000",
            "user@example.com",
            "John Doe",
            "invalid_status",
        );
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_uuid_valid() {
        assert!(validate_uuid("550e8400-e29b-41d4-a716-446655440000").is_ok());
    }

    #[test]
    fn test_validate_uuid_invalid_format() {
        assert!(validate_uuid("not-a-uuid").is_err());
        assert!(validate_uuid("550e8400-e29b-41d4-a716").is_err());
    }

    #[test]
    fn test_validate_uuid_empty() {
        assert!(validate_uuid("").is_err());
    }

    #[test]
    fn test_validate_stored_name_valid() {
        assert!(validate_stored_name("John Doe").is_ok());
        assert!(validate_stored_name("Jean-Pierre").is_ok());
        assert!(validate_stored_name("O'Brien").is_ok());
    }

    #[test]
    fn test_validate_stored_name_invalid() {
        assert!(validate_stored_name("").is_err());
        assert!(validate_stored_name(&"a".repeat(257)).is_err());
    }

    #[test]
    fn test_validate_stored_name_null_bytes() {
        assert!(validate_stored_name("Name\0with\0null").is_err());
    }

    #[test]
    fn test_validate_subscription_status_valid() {
        assert!(validate_subscription_status("pending").is_ok());
        assert!(validate_subscription_status("confirmed").is_ok());
    }

    #[test]
    fn test_validate_subscription_status_invalid() {
        assert!(validate_subscription_status("invalid").is_err());
        assert!(validate_subscription_status("").is_err());
    }

    #[test]
    fn test_validate_subscribers_batch_valid() {
        let subscribers = vec![
            (
                "550e8400-e29b-41d4-a716-446655440000".to_string(),
                "user1@example.com".to_string(),
                "User One".to_string(),
                "confirmed".to_string(),
            ),
            (
                "650e8400-e29b-41d4-a716-446655440001".to_string(),
                "user2@example.com".to_string(),
                "User Two".to_string(),
                "pending".to_string(),
            ),
        ];
        assert!(validate_subscribers_batch(&subscribers).is_ok());
    }

    #[test]
    fn test_validate_subscribers_batch_invalid() {
        let subscribers = vec![
            (
                "550e8400-e29b-41d4-a716-446655440000".to_string(),
                "user1@example.com".to_string(),
                "User One".to_string(),
                "confirmed".to_string(),
            ),
            (
                "invalid-uuid".to_string(),
                "user2@example.com".to_string(),
                "User Two".to_string(),
                "pending".to_string(),
            ),
        ];
        let result = validate_subscribers_batch(&subscribers);
        assert!(result.is_err());
        if let Err((idx, _)) = result {
            assert_eq!(idx, 1);
        }
    }
}
