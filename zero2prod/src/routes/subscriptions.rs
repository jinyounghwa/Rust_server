use actix_web::{web, HttpResponse};
use chrono::Utc;
use serde::Deserialize;
use sqlx::PgPool;
use uuid::Uuid;
use crate::validators::{is_valid_email, is_valid_name};
use crate::email_client::EmailClient;
use crate::confirmation_token::ConfirmationToken;
use crate::error::{AppError, DatabaseError, EmailError, ErrorContext};
use crate::request_logging::{RequestMetadata, FailedRequest, RequestFailureLogger, AuditLog};

#[derive(Deserialize)]
pub struct FormData {
    name: Option<String>,
    email: Option<String>,
}

pub async fn subscribe(
    form: web::Form<FormData>,
    pool: web::Data<PgPool>,
    email_client: web::Data<EmailClient>,
) -> Result<HttpResponse, AppError> {
    let error_context = ErrorContext::new("subscription_creation");

    // Validate name
    let name = form.name.as_ref()
        .ok_or_else(|| {
            let error = AppError::Validation(
                crate::error::ValidationError::EmptyField("name".to_string())
            );

            // 검증 실패 감사 로그
            let audit_log = AuditLog::new(
                "VALIDATE_INPUT".to_string(),
                "subscription".to_string(),
                "FAILURE".to_string(),
                "Missing required field: name".to_string(),
            );
            RequestFailureLogger::log_audit(&audit_log);

            error
        })?;
    let name = is_valid_name(name)
        .map_err(|e| {
            // 검증 실패 감사 로그
            let audit_log = AuditLog::new(
                "VALIDATE_NAME".to_string(),
                "subscription".to_string(),
                "FAILURE".to_string(),
                format!("Name validation failed: {}", e),
            );
            RequestFailureLogger::log_audit(&audit_log);
            AppError::Validation(e)
        })?;

    // Validate email
    let email = form.email.as_ref()
        .ok_or_else(|| {
            // 검증 실패 감사 로그
            let audit_log = AuditLog::new(
                "VALIDATE_INPUT".to_string(),
                "subscription".to_string(),
                "FAILURE".to_string(),
                "Missing required field: email".to_string(),
            );
            RequestFailureLogger::log_audit(&audit_log);

            AppError::Validation(
                crate::error::ValidationError::EmptyField("email".to_string())
            )
        })?;
    let email = is_valid_email(email)
        .map_err(|e| {
            // 검증 실패 감사 로그
            let audit_log = AuditLog::new(
                "VALIDATE_EMAIL".to_string(),
                "subscription".to_string(),
                "FAILURE".to_string(),
                format!("Email validation failed: {}", e),
            );
            RequestFailureLogger::log_audit(&audit_log);
            AppError::Validation(e)
        })?;

    tracing::info!(
        request_id = %error_context.request_id,
        "Processing new subscription (sensitive data redacted)"
    );

    let subscriber_id = Uuid::new_v4();

    // Insert subscriber into database
    create_subscriber(&pool, subscriber_id, &email, &name, &error_context).await?;

    // Generate and save confirmation token
    let confirmation_token = ConfirmationToken::new(subscriber_id);
    save_confirmation_token(&pool, subscriber_id, &confirmation_token, &error_context).await?;

    // Send confirmation email
    send_confirmation_email_flow(
        email_client.get_ref(),
        &email,
        &name,
        &confirmation_token,
        &error_context,
    )
    .await?;

    tracing::info!(
        request_id = %error_context.request_id,
        subscriber_id = %subscriber_id,
        "Subscription created successfully"
    );

    Ok(HttpResponse::Ok().finish())
}

/// Creates a new subscriber in the database with proper error handling
async fn create_subscriber(
    pool: &web::Data<PgPool>,
    subscriber_id: Uuid,
    email: &str,
    name: &str,
    context: &ErrorContext,
) -> Result<(), AppError> {
    sqlx::query(
        "INSERT INTO subscriptions (id, email, name, subscribed_at, status) VALUES ($1, $2, $3, $4, $5)"
    )
    .bind(subscriber_id)
    .bind(email)
    .bind(name)
    .bind(Utc::now())
    .bind("pending")
    .execute(pool.get_ref())
    .await
    .map_err(|e| {
        let error_str = e.to_string();
        let error = AppError::from(e);
        context.log_error(&error);

        // 데이터베이스 오류 상세 기록
        let request_metadata = RequestMetadata::new(
            context.request_id.clone(),
            "POST".to_string(),
            "/subscriptions".to_string(),
        );

        let is_duplicate = error_str.contains("duplicate key");
        let error_message = if is_duplicate {
            "Email already registered".to_string()
        } else {
            format!("Database error: {}", error_str)
        };

        let failed_request = FailedRequest::new(
            request_metadata,
            "DatabaseError".to_string(),
            error_message.clone(),
            if is_duplicate { "DUPLICATE_ENTRY" } else { "DATABASE_ERROR" }.to_string(),
            if is_duplicate { 409 } else { 500 },
        )
        .with_retryable(!is_duplicate && error_str.contains("pool"));

        RequestFailureLogger::log_failed_request(&failed_request);

        // 데이터베이스 오류 감사 로그
        let audit_log = AuditLog::new(
            "CREATE_SUBSCRIBER".to_string(),
            "subscription".to_string(),
            "FAILURE".to_string(),
            error_message,
        )
        .with_resource_id(subscriber_id.to_string());
        RequestFailureLogger::log_audit(&audit_log);

        error
    })?;

    tracing::info!(
        request_id = %context.request_id,
        subscriber_id = %subscriber_id,
        "New subscriber saved successfully"
    );

    // 성공 감사 로그
    let audit_log = AuditLog::new(
        "CREATE_SUBSCRIBER".to_string(),
        "subscription".to_string(),
        "SUCCESS".to_string(),
        "Subscriber created successfully".to_string(),
    )
    .with_resource_id(subscriber_id.to_string());
    RequestFailureLogger::log_audit(&audit_log);

    Ok(())
}

/// Saves confirmation token to database
async fn save_confirmation_token(
    pool: &web::Data<PgPool>,
    subscriber_id: Uuid,
    token: &ConfirmationToken,
    context: &ErrorContext,
) -> Result<(), AppError> {
    sqlx::query(
        r#"
        INSERT INTO subscription_tokens
        (subscription_token, subscriber_id, created_at, expires_at)
        VALUES ($1, $2, $3, $4)
        "#
    )
    .bind(token.token())
    .bind(subscriber_id.to_string())
    .bind(token.created_at())
    .bind(token.expires_at())
    .execute(pool.get_ref())
    .await
    .map_err(|e| {
        let error = AppError::Database(DatabaseError::UnexpectedError(
            format!("Failed to save confirmation token: {}", e)
        ));
        context.log_error(&error);
        error
    })?;

    tracing::info!(
        request_id = %context.request_id,
        subscriber_id = %subscriber_id,
        "Confirmation token saved successfully"
    );

    Ok(())
}

/// Sends confirmation email with proper error handling
async fn send_confirmation_email_flow(
    email_client: &EmailClient,
    recipient_email: &str,
    name: &str,
    token: &ConfirmationToken,
    context: &ErrorContext,
) -> Result<(), AppError> {
    let confirmation_link = format!(
        "http://localhost:8000/subscriptions/confirm?token={}",
        token.token()
    );

    let html_content = format!(
        r#"
        <h1>Welcome {}!</h1>
        <p>Please confirm your email subscription by clicking the link below:</p>
        <a href="{}">Confirm Subscription</a>
        <p>This link will expire in 24 hours.</p>
        "#,
        name, confirmation_link
    );

    send_confirmation_email(email_client, recipient_email, &html_content)
        .await
        .map_err(|e| {
            let error = AppError::Email(e.clone());
            context.log_error(&error);

            // 이메일 서비스 오류 상세 기록
            let request_metadata = RequestMetadata::new(
                context.request_id.clone(),
                "POST".to_string(),
                "/subscriptions".to_string(),
            );

            let error_message = format!("Failed to send confirmation email: {}", e);
            let mut failed_request = FailedRequest::new(
                request_metadata,
                "EmailError".to_string(),
                error_message.clone(),
                "EMAIL_SERVICE_ERROR".to_string(),
                503,  // Service Unavailable
            )
            .with_retryable(true);  // 이메일 서비스 오류는 일반적으로 재시도 가능

            RequestFailureLogger::log_failed_request(&failed_request);

            // 이메일 오류 감사 로그
            let audit_log = AuditLog::new(
                "SEND_CONFIRMATION_EMAIL".to_string(),
                "email".to_string(),
                "FAILURE".to_string(),
                error_message,
            );
            RequestFailureLogger::log_audit(&audit_log);

            error
        })?;

    tracing::info!(
        request_id = %context.request_id,
        "Confirmation email sent successfully"
    );

    // 이메일 전송 성공 감사 로그
    let audit_log = AuditLog::new(
        "SEND_CONFIRMATION_EMAIL".to_string(),
        "email".to_string(),
        "SUCCESS".to_string(),
        "Confirmation email sent successfully".to_string(),
    );
    RequestFailureLogger::log_audit(&audit_log);

    Ok(())
}

async fn send_confirmation_email(
    email_client: &EmailClient,
    recipient_email: &str,
    html_content: &str,
) -> Result<(), EmailError> {
    email_client
        .send_email(
            recipient_email,
            "Please confirm your subscription",
            html_content,
        )
        .await
}
