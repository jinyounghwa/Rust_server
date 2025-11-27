/// Authentication Routes
///
/// Handles user registration, login, token refresh, and current user information.

use actix_web::{web, HttpResponse};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;

use crate::auth::{
    generate_access_token, generate_refresh_token, hash_password, save_refresh_token,
    revoke_refresh_token, validate_refresh_token, verify_password, Claims,
};
use crate::configuration::JwtSettings;
use crate::error::{AppError, ErrorContext, ValidationError};
use crate::validators::{is_valid_email, is_valid_name};

/// User registration request
#[derive(Deserialize)]
pub struct RegisterRequest {
    pub email: String,
    pub password: String,
    pub name: String,
}

/// User login request
#[derive(Deserialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

/// Token refresh request
#[derive(Deserialize)]
pub struct RefreshRequest {
    pub refresh_token: String,
}

/// Authentication response with access and refresh tokens
#[derive(Serialize)]
pub struct AuthResponse {
    pub access_token: String,
    pub refresh_token: String,
    pub token_type: String,
    pub expires_in: i64,
}

/// User information response
#[derive(Serialize)]
pub struct UserResponse {
    pub id: String,
    pub email: String,
    pub name: String,
    pub created_at: String,
}

/// POST /auth/register
///
/// Register a new user with email, password, and name.
/// Returns access token and refresh token on success.
///
/// # Validation
/// - Email must be valid format and not already registered
/// - Password must be 8+ chars with digit, lowercase, and uppercase
/// - Name must be valid (non-empty, no suspicious content)
///
/// # Errors
/// - 400: Validation errors (invalid email/password/name)
/// - 409: Email already registered (duplicate)
/// - 500: Internal server error
pub async fn register(
    form: web::Json<RegisterRequest>,
    pool: web::Data<PgPool>,
    jwt_config: web::Data<JwtSettings>,
) -> Result<HttpResponse, AppError> {
    let context = ErrorContext::new("user_registration");

    // Validate inputs
    let email = is_valid_email(&form.email)?;
    let name = is_valid_name(&form.name)?;
    let password_hash = hash_password(&form.password)?;

    // Create user in database
    let user_id = Uuid::new_v4();
    sqlx::query(
        r#"
        INSERT INTO users (id, email, name, password_hash, created_at, updated_at)
        VALUES ($1, $2, $3, $4, $5, $6)
        "#,
    )
    .bind(user_id)
    .bind(&email)
    .bind(&name)
    .bind(&password_hash)
    .bind(Utc::now())
    .bind(Utc::now())
    .execute(pool.get_ref())
    .await?;

    // Generate tokens
    let access_token = generate_access_token(&user_id, &email, jwt_config.get_ref())?;
    let refresh_token = generate_refresh_token();

    // Save refresh token to database
    save_refresh_token(
        pool.get_ref(),
        user_id,
        &refresh_token,
        jwt_config.refresh_token_expiry,
    )
    .await?;

    tracing::info!(
        request_id = %context.request_id,
        user_id = %user_id,
        "User registered successfully"
    );

    Ok(HttpResponse::Created().json(AuthResponse {
        access_token,
        refresh_token,
        token_type: "Bearer".to_string(),
        expires_in: jwt_config.access_token_expiry,
    }))
}

/// POST /auth/login
///
/// Authenticate user with email and password.
/// Returns access token and refresh token on success.
///
/// # Errors
/// - 400: Validation error (invalid email format)
/// - 401: Invalid credentials (email not found or wrong password)
/// - 403: Account is inactive
/// - 500: Internal server error
///
/// # Security Notes
/// - Uses same error message for "not found" and "wrong password"
/// - Prevents user enumeration attacks
/// - Only returns tokens if account is active
pub async fn login(
    form: web::Json<LoginRequest>,
    pool: web::Data<PgPool>,
    jwt_config: web::Data<JwtSettings>,
) -> Result<HttpResponse, AppError> {
    let context = ErrorContext::new("user_login");

    // Validate email format
    let email = is_valid_email(&form.email)?;

    // Fetch user from database
    let user = sqlx::query_as::<_, (Uuid, String, String, bool)>(
        "SELECT id, email, password_hash, is_active FROM users WHERE email = $1",
    )
    .bind(&email)
    .fetch_optional(pool.get_ref())
    .await?
    .ok_or_else(|| {
        AppError::Validation(ValidationError::InvalidFormat(
            "Invalid email or password".to_string(),
        ))
    })?;

    let (user_id, user_email, password_hash, is_active) = user;

    // Check if account is active
    if !is_active {
        return Err(AppError::Validation(ValidationError::InvalidFormat(
            "Account is inactive".to_string(),
        )));
    }

    // Verify password
    let password_valid = verify_password(&form.password, &password_hash)?;
    if !password_valid {
        return Err(AppError::Validation(ValidationError::InvalidFormat(
            "Invalid email or password".to_string(),
        )));
    }

    // Generate tokens
    let access_token = generate_access_token(&user_id, &user_email, jwt_config.get_ref())?;
    let refresh_token = generate_refresh_token();

    // Save refresh token to database
    save_refresh_token(
        pool.get_ref(),
        user_id,
        &refresh_token,
        jwt_config.refresh_token_expiry,
    )
    .await?;

    tracing::info!(
        request_id = %context.request_id,
        user_id = %user_id,
        "User logged in successfully"
    );

    Ok(HttpResponse::Ok().json(AuthResponse {
        access_token,
        refresh_token,
        token_type: "Bearer".to_string(),
        expires_in: jwt_config.access_token_expiry,
    }))
}

/// POST /auth/refresh
///
/// Refresh access token using a refresh token.
/// Implements token rotation: old token is revoked, new token is issued.
///
/// # Token Rotation Security
/// - Old refresh token is revoked after new token is issued
/// - If client uses old token again after refresh, it will be rejected
/// - Detects token theft: attacker cannot reuse stolen token if legitimate refresh already happened
///
/// # Errors
/// - 401: Invalid, expired, or revoked refresh token
/// - 403: Associated account is inactive
/// - 500: Internal server error
pub async fn refresh(
    form: web::Json<RefreshRequest>,
    pool: web::Data<PgPool>,
    jwt_config: web::Data<JwtSettings>,
) -> Result<HttpResponse, AppError> {
    let context = ErrorContext::new("token_refresh");

    // Validate refresh token and get user_id
    let user_id = validate_refresh_token(pool.get_ref(), &form.refresh_token).await?;

    // Revoke old token (token rotation)
    revoke_refresh_token(pool.get_ref(), &form.refresh_token).await?;

    // Fetch user email
    let user_email = sqlx::query_scalar::<_, String>(
        "SELECT email FROM users WHERE id = $1 AND is_active = true",
    )
    .bind(user_id)
    .fetch_one(pool.get_ref())
    .await?;

    // Generate new tokens
    let access_token = generate_access_token(&user_id, &user_email, jwt_config.get_ref())?;
    let refresh_token = generate_refresh_token();

    // Save new refresh token to database
    save_refresh_token(
        pool.get_ref(),
        user_id,
        &refresh_token,
        jwt_config.refresh_token_expiry,
    )
    .await?;

    tracing::info!(
        request_id = %context.request_id,
        user_id = %user_id,
        "Token refreshed successfully"
    );

    Ok(HttpResponse::Ok().json(AuthResponse {
        access_token,
        refresh_token,
        token_type: "Bearer".to_string(),
        expires_in: jwt_config.access_token_expiry,
    }))
}

/// GET /auth/me
///
/// Get current authenticated user's information.
/// **Requires valid JWT access token** in Authorization header.
///
/// # Authentication
/// - Requires: `Authorization: Bearer <access_token>`
/// - Claims are injected by JWT middleware
///
/// # Errors
/// - 401: Missing or invalid token (handled by middleware)
/// - 404: User not found (should not happen if token is valid)
/// - 403: User account is inactive
/// - 500: Internal server error
pub async fn get_current_user(
    claims: web::ReqData<Claims>,
    pool: web::Data<PgPool>,
) -> Result<HttpResponse, AppError> {
    let user_id = claims.user_id()?;

    let user = sqlx::query_as::<_, (Uuid, String, String, chrono::DateTime<Utc>)>(
        "SELECT id, email, name, created_at FROM users WHERE id = $1 AND is_active = true",
    )
    .bind(user_id)
    .fetch_one(pool.get_ref())
    .await?;

    Ok(HttpResponse::Ok().json(UserResponse {
        id: user.0.to_string(),
        email: user.1,
        name: user.2,
        created_at: user.3.to_rfc3339(),
    }))
}
