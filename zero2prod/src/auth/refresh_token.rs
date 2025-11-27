/// Refresh Token Management
///
/// Handles secure refresh token generation, storage, validation, and revocation.
/// Refresh tokens are:
/// - Cryptographically secure random 64-byte strings
/// - Hashed with SHA-256 before storage (never store plaintext)
/// - Single-use with automatic revocation on refresh (token rotation)
/// - Database-backed for revocation support

use chrono::{Duration, Utc};
use rand::distributions::Alphanumeric;
use rand::{thread_rng, Rng};
use sha2::{Digest, Sha256};
use sqlx::PgPool;
use uuid::Uuid;

use crate::error::{AppError, ValidationError};

/// Generate a new cryptographically secure refresh token
///
/// Creates a 64-byte random token encoded as base62 characters.
/// The token is returned in plaintext (this is what the client stores).
/// The server stores only the SHA-256 hash.
pub fn generate_refresh_token() -> String {
    thread_rng()
        .sample_iter(&Alphanumeric)
        .take(64)
        .map(char::from)
        .collect()
}

/// Hash a refresh token using SHA-256
///
/// Never store plaintext tokens in the database.
fn hash_token(token: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(token.as_bytes());
    format!("{:x}", hasher.finalize())
}

/// Save a refresh token to the database
///
/// # Arguments
/// * `pool` - Database connection pool
/// * `user_id` - User ID that owns this token
/// * `token` - Plaintext refresh token
/// * `expiry_seconds` - Token lifetime in seconds
///
/// # Errors
/// Returns error if database operation fails
pub async fn save_refresh_token(
    pool: &PgPool,
    user_id: Uuid,
    token: &str,
    expiry_seconds: i64,
) -> Result<(), AppError> {
    let token_hash = hash_token(token);
    let expires_at = Utc::now() + Duration::seconds(expiry_seconds);

    sqlx::query(
        r#"
        INSERT INTO refresh_tokens (id, user_id, token_hash, expires_at, created_at)
        VALUES ($1, $2, $3, $4, $5)
        "#,
    )
    .bind(Uuid::new_v4())
    .bind(user_id)
    .bind(token_hash)
    .bind(expires_at)
    .bind(Utc::now())
    .execute(pool)
    .await?;

    Ok(())
}

/// Validate a refresh token
///
/// Checks:
/// 1. Token exists in database
/// 2. Token has not been revoked
/// 3. Token has not expired
///
/// # Arguments
/// * `pool` - Database connection pool
/// * `token` - Plaintext refresh token to validate
///
/// # Returns
/// User ID associated with the token if valid
///
/// # Errors
/// Returns error if token is invalid, revoked, or expired
pub async fn validate_refresh_token(pool: &PgPool, token: &str) -> Result<Uuid, AppError> {
    let token_hash = hash_token(token);

    let result = sqlx::query_as::<_, (Uuid, chrono::DateTime<Utc>, bool)>(
        r#"
        SELECT user_id, expires_at, is_revoked
        FROM refresh_tokens
        WHERE token_hash = $1
        "#,
    )
    .bind(&token_hash)
    .fetch_optional(pool)
    .await?;

    match result {
        None => {
            tracing::warn!("Refresh token not found in database");
            Err(AppError::Validation(ValidationError::InvalidFormat(
                "Invalid refresh token".to_string(),
            )))
        }
        Some((user_id, expires_at, is_revoked)) => {
            // Check if token is revoked
            if is_revoked {
                tracing::warn!(user_id = %user_id, "Attempt to use revoked refresh token");
                return Err(AppError::Validation(ValidationError::InvalidFormat(
                    "Token has been revoked".to_string(),
                )));
            }

            // Check if token has expired
            if expires_at < Utc::now() {
                tracing::info!(user_id = %user_id, "Refresh token expired");
                return Err(AppError::Validation(ValidationError::InvalidFormat(
                    "Token has expired".to_string(),
                )));
            }

            Ok(user_id)
        }
    }
}

/// Revoke a single refresh token
///
/// Used for token rotation - old token is revoked when new token is issued.
///
/// # Arguments
/// * `pool` - Database connection pool
/// * `token` - Plaintext refresh token to revoke
///
/// # Errors
/// Returns error if database operation fails
pub async fn revoke_refresh_token(pool: &PgPool, token: &str) -> Result<(), AppError> {
    let token_hash = hash_token(token);

    sqlx::query(
        r#"
        UPDATE refresh_tokens
        SET is_revoked = true, revoked_at = $1
        WHERE token_hash = $2
        "#,
    )
    .bind(Utc::now())
    .bind(token_hash)
    .execute(pool)
    .await?;

    Ok(())
}

/// Revoke all refresh tokens for a user
///
/// Useful for logout-all-devices functionality.
///
/// # Arguments
/// * `pool` - Database connection pool
/// * `user_id` - User whose tokens to revoke
///
/// # Errors
/// Returns error if database operation fails
pub async fn revoke_all_user_tokens(pool: &PgPool, user_id: Uuid) -> Result<(), AppError> {
    sqlx::query(
        r#"
        UPDATE refresh_tokens
        SET is_revoked = true, revoked_at = $1
        WHERE user_id = $2 AND is_revoked = false
        "#,
    )
    .bind(Utc::now())
    .bind(user_id)
    .execute(pool)
    .await?;

    tracing::info!(user_id = %user_id, "All refresh tokens revoked for user");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_refresh_token() {
        let token = generate_refresh_token();

        // Token should be 64 characters
        assert_eq!(token.len(), 64);
        // Token should be alphanumeric
        assert!(token.chars().all(|c| c.is_alphanumeric()));
    }

    #[test]
    fn test_token_hashing() {
        let token = generate_refresh_token();
        let hash1 = hash_token(&token);
        let hash2 = hash_token(&token);

        // Same token should produce same hash
        assert_eq!(hash1, hash2);
        // Hash should not equal plaintext
        assert_ne!(token, hash1);
        // Hash should be 64 chars (SHA-256 hex)
        assert_eq!(hash1.len(), 64);
    }

    #[test]
    fn test_different_tokens_different_hashes() {
        let token1 = generate_refresh_token();
        let token2 = generate_refresh_token();

        let hash1 = hash_token(&token1);
        let hash2 = hash_token(&token2);

        // Different tokens should produce different hashes
        assert_ne!(hash1, hash2);
    }
}
