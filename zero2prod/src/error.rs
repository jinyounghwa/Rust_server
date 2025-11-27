/// Comprehensive Error Handling Module
///
/// This module provides a unified error handling system for the entire application.
/// It covers:
/// 1. Control Flow Errors (Result-based)
/// 2. Operator/System Errors (HTTP responses with structured context)
/// 3. Custom Error Trait Implementation
/// 4. Domain-Specific Error Types (avoiding ball of mud)
/// 5. Structured Error Logging with Context

use actix_web::{error::ResponseError, http::StatusCode, HttpResponse};
use std::error::Error as StdError;
use std::fmt;

/// ============================================================================
/// 1. DOMAIN-SPECIFIC ERROR TYPES (Avoiding Ball of Mud)
/// ============================================================================

/// Validation errors for input data
#[derive(Debug, Clone)]
pub enum ValidationError {
    EmptyField(String),
    TooShort(String, usize),
    TooLong(String, usize),
    InvalidFormat(String),
    SuspiciousContent(String),
    PossibleSQLInjection,
}

impl fmt::Display for ValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ValidationError::EmptyField(field) => write!(f, "{} is empty", field),
            ValidationError::TooShort(field, min) => {
                write!(f, "{} is too short (minimum {} characters)", field, min)
            }
            ValidationError::TooLong(field, max) => {
                write!(f, "{} is too long (maximum {} characters)", field, max)
            }
            ValidationError::InvalidFormat(field) => write!(f, "{} has invalid format", field),
            ValidationError::SuspiciousContent(field) => {
                write!(f, "{} contains suspicious content", field)
            }
            ValidationError::PossibleSQLInjection => {
                write!(f, "input contains potentially dangerous SQL patterns")
            }
        }
    }
}

impl StdError for ValidationError {}

/// Database operation errors
#[derive(Debug)]
pub enum DatabaseError {
    UniqueConstraintViolation(String),
    NotFound(String),
    QueryExecution(String),
    ConnectionPool(String),
    UnexpectedError(String),
}

impl fmt::Display for DatabaseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DatabaseError::UniqueConstraintViolation(msg) => {
                write!(f, "Duplicate entry: {}", msg)
            }
            DatabaseError::NotFound(msg) => write!(f, "Not found: {}", msg),
            DatabaseError::QueryExecution(msg) => write!(f, "Query error: {}", msg),
            DatabaseError::ConnectionPool(msg) => write!(f, "Database connection error: {}", msg),
            DatabaseError::UnexpectedError(msg) => write!(f, "Database error: {}", msg),
        }
    }
}

impl StdError for DatabaseError {}

/// Email service errors
#[derive(Debug, Clone)]
pub enum EmailError {
    SendFailed(String),
    InvalidRecipient(String),
    ServiceUnavailable(String),
    ConfigurationError(String),
}

impl fmt::Display for EmailError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            EmailError::SendFailed(msg) => write!(f, "Failed to send email: {}", msg),
            EmailError::InvalidRecipient(msg) => write!(f, "Invalid recipient: {}", msg),
            EmailError::ServiceUnavailable(msg) => {
                write!(f, "Email service unavailable: {}", msg)
            }
            EmailError::ConfigurationError(msg) => write!(f, "Email config error: {}", msg),
        }
    }
}

impl StdError for EmailError {}

/// Configuration errors
#[derive(Debug)]
pub enum ConfigError {
    MissingRequired(String),
    InvalidValue(String),
    ParseError(String),
}

impl fmt::Display for ConfigError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ConfigError::MissingRequired(msg) => write!(f, "Missing required config: {}", msg),
            ConfigError::InvalidValue(msg) => write!(f, "Invalid config value: {}", msg),
            ConfigError::ParseError(msg) => write!(f, "Config parse error: {}", msg),
        }
    }
}

impl StdError for ConfigError {}

/// Authentication and authorization errors
#[derive(Debug)]
pub enum AuthError {
    InvalidCredentials,
    TokenExpired,
    TokenInvalid,
    MissingToken,
    AccountInactive,
}

impl fmt::Display for AuthError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AuthError::InvalidCredentials => write!(f, "Invalid credentials"),
            AuthError::TokenExpired => write!(f, "Token has expired"),
            AuthError::TokenInvalid => write!(f, "Invalid token"),
            AuthError::MissingToken => write!(f, "Missing authentication token"),
            AuthError::AccountInactive => write!(f, "Account is inactive"),
        }
    }
}

impl StdError for AuthError {}

/// ============================================================================
/// 2. UNIFIED APPLICATION ERROR TYPE
/// ============================================================================

/// Central error type that all application errors map to
/// This is used for control flow within the application
#[derive(Debug)]
pub enum AppError {
    Validation(ValidationError),
    Database(DatabaseError),
    Email(EmailError),
    Auth(AuthError),
    Config(ConfigError),
    Internal(String),
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AppError::Validation(e) => write!(f, "{}", e),
            AppError::Database(e) => write!(f, "{}", e),
            AppError::Email(e) => write!(f, "{}", e),
            AppError::Auth(e) => write!(f, "{}", e),
            AppError::Config(e) => write!(f, "{}", e),
            AppError::Internal(msg) => write!(f, "Internal error: {}", msg),
        }
    }
}

impl StdError for AppError {}

// ============================================================================
// FROM IMPLEMENTATIONS (Control Flow Error Conversion)
// ============================================================================

impl From<ValidationError> for AppError {
    fn from(err: ValidationError) -> Self {
        AppError::Validation(err)
    }
}

impl From<DatabaseError> for AppError {
    fn from(err: DatabaseError) -> Self {
        AppError::Database(err)
    }
}

impl From<EmailError> for AppError {
    fn from(err: EmailError) -> Self {
        AppError::Email(err)
    }
}

impl From<AuthError> for AppError {
    fn from(err: AuthError) -> Self {
        AppError::Auth(err)
    }
}

impl From<ConfigError> for AppError {
    fn from(err: ConfigError) -> Self {
        AppError::Config(err)
    }
}

impl From<sqlx::Error> for AppError {
    fn from(err: sqlx::Error) -> Self {
        let error_msg = err.to_string();

        if error_msg.contains("duplicate key") || error_msg.contains("unique constraint") {
            AppError::Database(DatabaseError::UniqueConstraintViolation(
                "Email already registered".to_string(),
            ))
        } else if error_msg.contains("no rows") {
            AppError::Database(DatabaseError::NotFound(
                "Record not found".to_string(),
            ))
        } else if error_msg.contains("pool") || error_msg.contains("connect") {
            AppError::Database(DatabaseError::ConnectionPool(error_msg))
        } else {
            AppError::Database(DatabaseError::UnexpectedError(error_msg))
        }
    }
}

impl From<String> for AppError {
    fn from(msg: String) -> Self {
        AppError::Internal(msg)
    }
}

impl From<&str> for AppError {
    fn from(msg: &str) -> Self {
        AppError::Internal(msg.to_string())
    }
}

// ============================================================================
// 3. HTTP RESPONSE MAPPING (Operator/System Error Handling)
// ============================================================================

/// Error response structure for HTTP responses
#[derive(Debug, serde::Serialize)]
pub struct ErrorResponse {
    /// Unique error ID for tracking (request ID or trace ID)
    pub error_id: String,
    /// Human-readable error message
    pub message: String,
    /// Error code for client-side handling
    pub code: String,
    /// HTTP status code
    pub status: u16,
    /// Timestamp when error occurred
    pub timestamp: String,
}

impl ErrorResponse {
    /// Create a new error response
    pub fn new(error_id: String, message: String, code: String, status: u16) -> Self {
        Self {
            error_id,
            message,
            code,
            status,
            timestamp: chrono::Utc::now().to_rfc3339(),
        }
    }
}

/// Trait for converting errors to HTTP responses with proper logging
pub trait ErrorHandler {
    fn error_response(&self, request_id: &str) -> (StatusCode, ErrorResponse);
    fn log_error(&self, request_id: &str);
}

impl ErrorHandler for AppError {
    fn error_response(&self, request_id: &str) -> (StatusCode, ErrorResponse) {
        let (status, code, message) = match self {
            // Validation errors -> 400 Bad Request
            AppError::Validation(e) => (
                StatusCode::BAD_REQUEST,
                "VALIDATION_ERROR".to_string(),
                e.to_string(),
            ),

            // Database errors -> appropriate HTTP status
            AppError::Database(e) => match e {
                DatabaseError::UniqueConstraintViolation(_) => (
                    StatusCode::CONFLICT,
                    "DUPLICATE_ENTRY".to_string(),
                    e.to_string(),
                ),
                DatabaseError::NotFound(_) => (
                    StatusCode::NOT_FOUND,
                    "NOT_FOUND".to_string(),
                    e.to_string(),
                ),
                DatabaseError::ConnectionPool(_) => (
                    StatusCode::SERVICE_UNAVAILABLE,
                    "SERVICE_UNAVAILABLE".to_string(),
                    "Database service temporarily unavailable".to_string(),
                ),
                _ => (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "DATABASE_ERROR".to_string(),
                    "Database error occurred".to_string(),
                ),
            },

            // Email errors -> 503 Service Unavailable
            AppError::Email(_) => (
                StatusCode::SERVICE_UNAVAILABLE,
                "EMAIL_SERVICE_ERROR".to_string(),
                "Email service temporarily unavailable".to_string(),
            ),

            // Authentication errors -> 401 Unauthorized or 403 Forbidden
            AppError::Auth(e) => match e {
                AuthError::InvalidCredentials => (
                    StatusCode::UNAUTHORIZED,
                    "INVALID_CREDENTIALS".to_string(),
                    "Invalid credentials".to_string(),
                ),
                AuthError::TokenExpired | AuthError::TokenInvalid => (
                    StatusCode::UNAUTHORIZED,
                    "TOKEN_INVALID".to_string(),
                    "Invalid or expired token".to_string(),
                ),
                AuthError::MissingToken => (
                    StatusCode::UNAUTHORIZED,
                    "MISSING_TOKEN".to_string(),
                    "Missing authentication token".to_string(),
                ),
                AuthError::AccountInactive => (
                    StatusCode::FORBIDDEN,
                    "ACCOUNT_INACTIVE".to_string(),
                    "Account is inactive".to_string(),
                ),
            },

            // Config errors -> 500 Internal Server Error
            AppError::Config(_) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "CONFIG_ERROR".to_string(),
                "Server configuration error".to_string(),
            ),

            // Internal errors -> 500 Internal Server Error
            AppError::Internal(_) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "INTERNAL_ERROR".to_string(),
                "Internal server error".to_string(),
            ),
        };

        let error_response = ErrorResponse::new(
            request_id.to_string(),
            message,
            code,
            status.as_u16(),
        );

        (status, error_response)
    }

    fn log_error(&self, request_id: &str) {
        match self {
            AppError::Validation(e) => {
                tracing::warn!(
                    request_id = request_id,
                    error = %e,
                    "Validation error"
                );
            }
            AppError::Database(DatabaseError::UniqueConstraintViolation(_)) => {
                tracing::warn!(
                    request_id = request_id,
                    error = %self,
                    "Duplicate entry attempt"
                );
            }
            AppError::Database(e) => {
                tracing::error!(
                    request_id = request_id,
                    error = %e,
                    "Database error"
                );
            }
            AppError::Email(e) => {
                tracing::error!(
                    request_id = request_id,
                    error = %e,
                    "Email service error"
                );
            }
            AppError::Auth(e) => {
                match e {
                    AuthError::InvalidCredentials => {
                        tracing::warn!(
                            request_id = request_id,
                            error = %e,
                            "Invalid credentials attempt"
                        );
                    }
                    _ => {
                        tracing::warn!(
                            request_id = request_id,
                            error = %e,
                            "Authentication error"
                        );
                    }
                }
            }
            AppError::Config(e) => {
                tracing::error!(
                    request_id = request_id,
                    error = %e,
                    "Configuration error"
                );
            }
            AppError::Internal(msg) => {
                tracing::error!(
                    request_id = request_id,
                    error = %msg,
                    "Internal error"
                );
            }
        }
    }
}

/// Implement ResponseError for Actix-web integration
impl ResponseError for AppError {
    fn error_response(&self) -> HttpResponse {
        let request_id = uuid::Uuid::new_v4().to_string();
        self.log_error(&request_id);

        let (status, error_response) = <Self as ErrorHandler>::error_response(self, &request_id);

        HttpResponse::build(status).json(error_response)
    }

    fn status_code(&self) -> StatusCode {
        match self {
            AppError::Validation(_) => StatusCode::BAD_REQUEST,
            AppError::Database(e) => match e {
                DatabaseError::UniqueConstraintViolation(_) => StatusCode::CONFLICT,
                DatabaseError::NotFound(_) => StatusCode::NOT_FOUND,
                DatabaseError::ConnectionPool(_) => StatusCode::SERVICE_UNAVAILABLE,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            },
            AppError::Email(_) => StatusCode::SERVICE_UNAVAILABLE,
            AppError::Auth(e) => match e {
                AuthError::AccountInactive => StatusCode::FORBIDDEN,
                _ => StatusCode::UNAUTHORIZED,
            },
            AppError::Config(_) => StatusCode::INTERNAL_SERVER_ERROR,
            AppError::Internal(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

// ============================================================================
// 4. HELPER FUNCTIONS FOR COMMON ERROR PATTERNS
// ============================================================================

/// Helper to create a result with validation error
#[inline]
pub fn validation_error(field: &str, msg: &str) -> Result<(), AppError> {
    Err(AppError::Validation(ValidationError::InvalidFormat(
        format!("{}: {}", field, msg),
    )))
}

/// Helper to create a database error result
#[inline]
pub fn db_error(msg: impl Into<String>) -> Result<(), AppError> {
    Err(AppError::Database(DatabaseError::UnexpectedError(
        msg.into(),
    )))
}

/// Helper to create an email error result
#[inline]
pub fn email_error(msg: impl Into<String>) -> Result<(), AppError> {
    Err(AppError::Email(EmailError::SendFailed(msg.into())))
}

// ============================================================================
// 5. ERROR CONTEXT ENRICHMENT
// ============================================================================

/// Error context for enhanced logging and debugging
#[derive(Debug, Clone)]
pub struct ErrorContext {
    pub request_id: String,
    pub user_id: Option<String>,
    pub operation: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

impl ErrorContext {
    pub fn new(operation: impl Into<String>) -> Self {
        Self {
            request_id: uuid::Uuid::new_v4().to_string(),
            user_id: None,
            operation: operation.into(),
            timestamp: chrono::Utc::now(),
        }
    }

    pub fn with_request_id(mut self, request_id: String) -> Self {
        self.request_id = request_id;
        self
    }

    pub fn with_user_id(mut self, user_id: String) -> Self {
        self.user_id = Some(user_id);
        self
    }

    pub fn log_error(&self, error: &AppError) {
        let context = serde_json::json!({
            "request_id": self.request_id,
            "operation": self.operation,
            "user_id": self.user_id,
            "timestamp": self.timestamp.to_rfc3339(),
        });

        match error {
            AppError::Validation(_) => {
                tracing::warn!(
                    error = %error,
                    context = ?context,
                    "Validation error"
                );
            }
            AppError::Database(_) => {
                tracing::error!(
                    error = %error,
                    context = ?context,
                    "Database error"
                );
            }
            AppError::Email(_) => {
                tracing::error!(
                    error = %error,
                    context = ?context,
                    "Email error"
                );
            }
            AppError::Auth(_) => {
                tracing::warn!(
                    error = %error,
                    context = ?context,
                    "Authentication error"
                );
            }
            AppError::Config(_) => {
                tracing::error!(
                    error = %error,
                    context = ?context,
                    "Configuration error"
                );
            }
            AppError::Internal(_) => {
                tracing::error!(
                    error = %error,
                    context = ?context,
                    "Internal error"
                );
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validation_error_display() {
        let err = ValidationError::EmptyField("email".to_string());
        assert_eq!(err.to_string(), "email is empty");
    }

    #[test]
    fn test_app_error_conversion() {
        let val_err = ValidationError::InvalidFormat("test".to_string());
        let app_err: AppError = val_err.into();
        match app_err {
            AppError::Validation(_) => (),
            _ => panic!("Expected Validation error"),
        }
    }

    #[test]
    fn test_error_response_creation() {
        let request_id = "test-123".to_string();
        let response = ErrorResponse::new(
            request_id.clone(),
            "Test error".to_string(),
            "TEST_ERROR".to_string(),
            400,
        );

        assert_eq!(response.error_id, request_id);
        assert_eq!(response.code, "TEST_ERROR");
        assert_eq!(response.status, 400);
    }

    #[test]
    fn test_error_context_creation() {
        let ctx = ErrorContext::new("test_operation");
        assert_eq!(ctx.operation, "test_operation");
        assert!(ctx.user_id.is_none());

        let ctx_with_user = ctx.with_user_id("user-123".to_string());
        assert_eq!(ctx_with_user.user_id, Some("user-123".to_string()));
    }
}
