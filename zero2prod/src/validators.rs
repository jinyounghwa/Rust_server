/// Input validators module - protects against invalid subscribers and attacks
/// Features:
/// 1. DoS Protection: Input length limits
/// 2. Data Theft Protection: Input sanitization
/// 3. Phishing Protection: Email validation
/// 4. SQL Injection Prevention: Query validation

use regex::Regex;
use lazy_static::lazy_static;
use crate::error::ValidationError;

const MAX_EMAIL_LENGTH: usize = 254; // RFC 5321
const MAX_NAME_LENGTH: usize = 256;  // Custom limit as per requirements
const MIN_EMAIL_LENGTH: usize = 5;   // Minimum valid email length
const MIN_NAME_LENGTH: usize = 1;    // At least one character

lazy_static! {
    // RFC 5322 simplified email regex (practical validation)
    static ref EMAIL_REGEX: Regex = Regex::new(
        r"^[a-zA-Z0-9.!#$%&'*+/=?^_`{|}~-]+@[a-zA-Z0-9](?:[a-zA-Z0-9-]{0,61}[a-zA-Z0-9])?(?:\.[a-zA-Z0-9](?:[a-zA-Z0-9-]{0,61}[a-zA-Z0-9])?)*$"
    ).unwrap();

    // Regex to detect potentially malicious SQL patterns
    static ref SQL_INJECTION_PATTERNS: [Regex; 6] = [
        // Union-based SQL injection
        Regex::new(r"(?i)\s+UNION\s+").unwrap(),
        // Comment-based injection
        Regex::new(r"(--|;|/\*|\*/|xp_|sp_)").unwrap(),
        // Stacked queries
        Regex::new(r"(?i);\s*(INSERT|UPDATE|DELETE|DROP|CREATE|ALTER)").unwrap(),
        // Time-based blind injection
        Regex::new(r"(?i)(SLEEP|WAITFOR|BENCHMARK|DBMS_LOCK)").unwrap(),
        // Boolean-based injection - quotes handled with character class
        Regex::new(r#"(?i)(\bOR\b|\bAND\b)\s*(['"][0-9]*['"]|[0-9]*)\s*=\s*(['"][0-9]*['"]|[0-9]*|True|False)"#).unwrap(),
        // Function-based injection
        Regex::new(r"(?i)(CAST|CONVERT|SUBSTRING|CONCAT|LOAD_FILE)").unwrap(),
    ];
}

/// Validates email address
/// - Checks format using RFC 5322 simplified regex
/// - Verifies length constraints
/// - Detects potential phishing patterns
pub fn is_valid_email(email: &str) -> Result<String, ValidationError> {
    let trimmed = email.trim();

    // Length validation - prevent DoS attacks with extremely long inputs
    if trimmed.is_empty() {
        return Err(ValidationError::EmptyField("email".to_string()));
    }

    if trimmed.len() < MIN_EMAIL_LENGTH {
        return Err(ValidationError::TooShort("email".to_string(), MIN_EMAIL_LENGTH));
    }

    if trimmed.len() > MAX_EMAIL_LENGTH {
        return Err(ValidationError::TooLong("email".to_string(), MAX_EMAIL_LENGTH));
    }

    // Format validation - RFC 5322 simplified
    if !EMAIL_REGEX.is_match(trimmed) {
        return Err(ValidationError::InvalidFormat("email".to_string()));
    }

    // Check for suspicious patterns (phishing protection)
    if has_suspicious_email_patterns(trimmed) {
        return Err(ValidationError::SuspiciousContent("email".to_string()));
    }

    // Check for SQL injection patterns in email
    if contains_sql_injection_patterns(trimmed) {
        return Err(ValidationError::PossibleSQLInjection);
    }

    Ok(trimmed.to_string())
}

/// Validates subscriber name
/// - Checks length constraints
/// - Validates against control characters
/// - Detects SQL injection patterns
pub fn is_valid_name(name: &str) -> Result<String, ValidationError> {
    let trimmed = name.trim();

    // Length validation - prevent DoS attacks
    if trimmed.is_empty() {
        return Err(ValidationError::EmptyField("name".to_string()));
    }

    if trimmed.len() < MIN_NAME_LENGTH {
        return Err(ValidationError::TooShort("name".to_string(), MIN_NAME_LENGTH));
    }

    if trimmed.len() > MAX_NAME_LENGTH {
        return Err(ValidationError::TooLong("name".to_string(), MAX_NAME_LENGTH));
    }

    // Check for control characters and suspicious content
    if has_suspicious_name_patterns(trimmed) {
        return Err(ValidationError::SuspiciousContent("name".to_string()));
    }

    // Check for SQL injection patterns
    if contains_sql_injection_patterns(trimmed) {
        return Err(ValidationError::PossibleSQLInjection);
    }

    Ok(trimmed.to_string())
}

/// Detects suspicious patterns in email addresses that might indicate phishing
fn has_suspicious_email_patterns(email: &str) -> bool {

    // Check for extremely long local part (before @) - phishing indicator
    if let Some(at_pos) = email.find('@') {
        let local_part = &email[..at_pos];
        if local_part.len() > 64 {
            return true;
        }
    }

    // Check for multiple @ symbols
    if email.matches('@').count() != 1 {
        return true;
    }

    // Check for null bytes
    if email.contains('\0') {
        return true;
    }

    false
}

/// Detects suspicious patterns in names
fn has_suspicious_name_patterns(name: &str) -> bool {
    // Check for null bytes (data theft protection)
    if name.contains('\0') {
        return true;
    }

    // Check for control characters
    if name.chars().any(|c| c.is_control()) {
        return true;
    }

    // Check for excessive special characters (potential injection)
    let special_char_count = name.chars()
        .filter(|c| !c.is_alphanumeric() && !c.is_whitespace() && *c != '-' && *c != '.' && *c != '_' && *c != '\'')
        .count();

    if special_char_count > 5 {
        return true;
    }

    false
}

/// Checks if input contains SQL injection patterns
fn contains_sql_injection_patterns(input: &str) -> bool {
    SQL_INJECTION_PATTERNS.iter().any(|pattern| pattern.is_match(input))
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_email() {
        assert!(is_valid_email("user@example.com").is_ok());
        assert!(is_valid_email("test.email@domain.co.uk").is_ok());
        assert!(is_valid_email("user+tag@example.com").is_ok());
    }

    #[test]
    fn test_invalid_email_format() {
        assert!(is_valid_email("invalid").is_err());
        assert!(is_valid_email("user@").is_err());
        assert!(is_valid_email("@example.com").is_err());
        assert!(is_valid_email("user@@example.com").is_err());
    }

    #[test]
    fn test_email_length_limits() {
        let too_long = format!("{}@example.com", "a".repeat(250));
        assert!(is_valid_email(&too_long).is_err());

        // Test valid short email
        assert!(is_valid_email("a@a.com").is_ok());

        // Test too short email (below MIN_EMAIL_LENGTH of 5)
        assert!(is_valid_email("a@b").is_err());
    }

    #[test]
    fn test_sql_injection_in_email() {
        assert!(is_valid_email("user' OR '1'='1@example.com").is_err());
        assert!(is_valid_email("user; DROP TABLE@example.com").is_err());
    }

    #[test]
    fn test_valid_name() {
        assert!(is_valid_name("John Doe").is_ok());
        assert!(is_valid_name("Jean-Pierre").is_ok());
        assert!(is_valid_name("O'Brien").is_ok());
    }

    #[test]
    fn test_name_length_limits() {
        let too_long = "a".repeat(257);
        assert!(is_valid_name(&too_long).is_err());

        assert!(is_valid_name("").is_err());
    }

    #[test]
    fn test_sql_injection_in_name() {
        assert!(is_valid_name("John'; DROP TABLE subscribers--").is_err());
        assert!(is_valid_name("Name UNION SELECT *").is_err());
    }

    #[test]
    fn test_control_characters() {
        assert!(is_valid_name("Name\0with\0null").is_err());
    }

    #[test]
    fn test_excessive_special_characters() {
        assert!(is_valid_name("!!!!!!@@@@").is_err());
    }
}
