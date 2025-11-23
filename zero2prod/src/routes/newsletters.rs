use actix_web::{web, HttpResponse};
use serde::Deserialize;
use sqlx::PgPool;
use crate::email_client::EmailClient;
use crate::error::{AppError, DatabaseError, ErrorContext};
use crate::request_logging::{RequestMetadata, FailedRequest, RequestFailureLogger, AuditLog};
use crate::data_validation::validate_subscriber_data;

#[derive(Deserialize)]
pub struct NewsletterData {
    subject: Option<String>,
    html_content: Option<String>,
}

#[derive(sqlx::FromRow)]
pub struct SubscriberData {
    pub id: String,
    pub email: String,
    pub name: String,
    pub status: String,
}

/// Send email to all subscribers (including unconfirmed)
pub async fn send_newsletter_to_all(
    form: web::Json<NewsletterData>,
    pool: web::Data<PgPool>,
    email_client: web::Data<EmailClient>,
) -> Result<HttpResponse, AppError> {
    let error_context = ErrorContext::new("newsletter_send_all");

    // Validate subject
    let subject = form.subject.as_ref()
        .ok_or_else(|| {
            let audit_log = AuditLog::new(
                "VALIDATE_INPUT".to_string(),
                "newsletter".to_string(),
                "FAILURE".to_string(),
                "Missing required field: subject".to_string(),
            );
            RequestFailureLogger::log_audit(&audit_log);

            AppError::Validation(
                crate::error::ValidationError::EmptyField("subject".to_string())
            )
        })?;

    // Validate HTML content
    let html_content = form.html_content.as_ref()
        .ok_or_else(|| {
            let audit_log = AuditLog::new(
                "VALIDATE_INPUT".to_string(),
                "newsletter".to_string(),
                "FAILURE".to_string(),
                "Missing required field: html_content".to_string(),
            );
            RequestFailureLogger::log_audit(&audit_log);

            AppError::Validation(
                crate::error::ValidationError::EmptyField("html_content".to_string())
            )
        })?;

    if subject.trim().is_empty() {
        let audit_log = AuditLog::new(
            "VALIDATE_SUBJECT".to_string(),
            "newsletter".to_string(),
            "FAILURE".to_string(),
            "Subject cannot be empty".to_string(),
        );
        RequestFailureLogger::log_audit(&audit_log);

        return Err(AppError::Validation(
            crate::error::ValidationError::EmptyField("subject".to_string())
        ));
    }

    if html_content.trim().is_empty() {
        let audit_log = AuditLog::new(
            "VALIDATE_CONTENT".to_string(),
            "newsletter".to_string(),
            "FAILURE".to_string(),
            "HTML content cannot be empty".to_string(),
        );
        RequestFailureLogger::log_audit(&audit_log);

        return Err(AppError::Validation(
            crate::error::ValidationError::EmptyField("html_content".to_string())
        ));
    }

    tracing::info!(
        request_id = %error_context.request_id,
        "Processing newsletter send to all subscribers"
    );

    // Fetch all subscribers
    let subscribers = get_all_subscribers(&pool, &error_context).await?;

    if subscribers.is_empty() {
        let audit_log = AuditLog::new(
            "SEND_NEWSLETTER".to_string(),
            "newsletter".to_string(),
            "SUCCESS".to_string(),
            "No subscribers found - newsletter not sent".to_string(),
        );
        RequestFailureLogger::log_audit(&audit_log);

        tracing::info!(
            request_id = %error_context.request_id,
            "No subscribers found"
        );
        return Ok(HttpResponse::Ok().json(serde_json::json!({
            "message": "No subscribers found",
            "sent_count": 0
        })));
    }

    // Send email to each subscriber
    let mut sent_count = 0;
    let mut failed_count = 0;

    for subscriber in subscribers {
        // Validate subscriber data before sending
        if let Err(validation_err) = validate_subscriber_data(
            &subscriber.id,
            &subscriber.email,
            &subscriber.name,
            &subscriber.status,
        ) {
            failed_count += 1;
            let audit_log = AuditLog::new(
                "SEND_NEWSLETTER".to_string(),
                "newsletter".to_string(),
                "FAILURE".to_string(),
                format!("Subscriber data validation failed: {}", validation_err),
            )
            .with_resource_id(subscriber.id.clone());
            RequestFailureLogger::log_audit(&audit_log);

            tracing::warn!(
                request_id = %error_context.request_id,
                email = %subscriber.email,
                error = %validation_err,
                "Subscriber data validation failed"
            );
            continue;
        }

        match email_client.send_email(
            &subscriber.email,
            subject,
            html_content,
        ).await {
            Ok(_) => {
                sent_count += 1;
                let audit_log = AuditLog::new(
                    "SEND_NEWSLETTER".to_string(),
                    "newsletter".to_string(),
                    "SUCCESS".to_string(),
                    format!("Newsletter sent to subscriber"),
                )
                .with_resource_id(subscriber.id.clone());
                RequestFailureLogger::log_audit(&audit_log);
            }
            Err(e) => {
                failed_count += 1;
                let audit_log = AuditLog::new(
                    "SEND_NEWSLETTER".to_string(),
                    "newsletter".to_string(),
                    "FAILURE".to_string(),
                    format!("Failed to send newsletter to {}: {}", subscriber.email, e),
                )
                .with_resource_id(subscriber.id.clone());
                RequestFailureLogger::log_audit(&audit_log);

                tracing::warn!(
                    request_id = %error_context.request_id,
                    email = %subscriber.email,
                    error = %e,
                    "Failed to send newsletter to subscriber"
                );
            }
        }
    }

    tracing::info!(
        request_id = %error_context.request_id,
        sent_count = sent_count,
        failed_count = failed_count,
        "Newsletter send to all completed"
    );

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "message": "Newsletter sent to all subscribers",
        "sent_count": sent_count,
        "failed_count": failed_count
    })))
}

/// Send email to only confirmed subscribers
pub async fn send_newsletter_to_confirmed(
    form: web::Json<NewsletterData>,
    pool: web::Data<PgPool>,
    email_client: web::Data<EmailClient>,
) -> Result<HttpResponse, AppError> {
    let error_context = ErrorContext::new("newsletter_send_confirmed");

    // Validate subject
    let subject = form.subject.as_ref()
        .ok_or_else(|| {
            let audit_log = AuditLog::new(
                "VALIDATE_INPUT".to_string(),
                "newsletter".to_string(),
                "FAILURE".to_string(),
                "Missing required field: subject".to_string(),
            );
            RequestFailureLogger::log_audit(&audit_log);

            AppError::Validation(
                crate::error::ValidationError::EmptyField("subject".to_string())
            )
        })?;

    // Validate HTML content
    let html_content = form.html_content.as_ref()
        .ok_or_else(|| {
            let audit_log = AuditLog::new(
                "VALIDATE_INPUT".to_string(),
                "newsletter".to_string(),
                "FAILURE".to_string(),
                "Missing required field: html_content".to_string(),
            );
            RequestFailureLogger::log_audit(&audit_log);

            AppError::Validation(
                crate::error::ValidationError::EmptyField("html_content".to_string())
            )
        })?;

    if subject.trim().is_empty() {
        let audit_log = AuditLog::new(
            "VALIDATE_SUBJECT".to_string(),
            "newsletter".to_string(),
            "FAILURE".to_string(),
            "Subject cannot be empty".to_string(),
        );
        RequestFailureLogger::log_audit(&audit_log);

        return Err(AppError::Validation(
            crate::error::ValidationError::EmptyField("subject".to_string())
        ));
    }

    if html_content.trim().is_empty() {
        let audit_log = AuditLog::new(
            "VALIDATE_CONTENT".to_string(),
            "newsletter".to_string(),
            "FAILURE".to_string(),
            "HTML content cannot be empty".to_string(),
        );
        RequestFailureLogger::log_audit(&audit_log);

        return Err(AppError::Validation(
            crate::error::ValidationError::EmptyField("html_content".to_string())
        ));
    }

    tracing::info!(
        request_id = %error_context.request_id,
        "Processing newsletter send to confirmed subscribers"
    );

    // Fetch only confirmed subscribers
    let subscribers = get_confirmed_subscribers(&pool, &error_context).await?;

    if subscribers.is_empty() {
        let audit_log = AuditLog::new(
            "SEND_NEWSLETTER".to_string(),
            "newsletter".to_string(),
            "SUCCESS".to_string(),
            "No confirmed subscribers found - newsletter not sent".to_string(),
        );
        RequestFailureLogger::log_audit(&audit_log);

        tracing::info!(
            request_id = %error_context.request_id,
            "No confirmed subscribers found"
        );
        return Ok(HttpResponse::Ok().json(serde_json::json!({
            "message": "No confirmed subscribers found",
            "sent_count": 0
        })));
    }

    // Send email to each confirmed subscriber
    let mut sent_count = 0;
    let mut failed_count = 0;

    for subscriber in subscribers {
        // Validate subscriber data before sending
        if let Err(validation_err) = validate_subscriber_data(
            &subscriber.id,
            &subscriber.email,
            &subscriber.name,
            &subscriber.status,
        ) {
            failed_count += 1;
            let audit_log = AuditLog::new(
                "SEND_NEWSLETTER".to_string(),
                "newsletter".to_string(),
                "FAILURE".to_string(),
                format!("Subscriber data validation failed: {}", validation_err),
            )
            .with_resource_id(subscriber.id.clone());
            RequestFailureLogger::log_audit(&audit_log);

            tracing::warn!(
                request_id = %error_context.request_id,
                email = %subscriber.email,
                error = %validation_err,
                "Subscriber data validation failed"
            );
            continue;
        }

        match email_client.send_email(
            &subscriber.email,
            subject,
            html_content,
        ).await {
            Ok(_) => {
                sent_count += 1;
                let audit_log = AuditLog::new(
                    "SEND_NEWSLETTER".to_string(),
                    "newsletter".to_string(),
                    "SUCCESS".to_string(),
                    format!("Newsletter sent to confirmed subscriber"),
                )
                .with_resource_id(subscriber.id.clone());
                RequestFailureLogger::log_audit(&audit_log);
            }
            Err(e) => {
                failed_count += 1;
                let audit_log = AuditLog::new(
                    "SEND_NEWSLETTER".to_string(),
                    "newsletter".to_string(),
                    "FAILURE".to_string(),
                    format!("Failed to send newsletter to {}: {}", subscriber.email, e),
                )
                .with_resource_id(subscriber.id.clone());
                RequestFailureLogger::log_audit(&audit_log);

                tracing::warn!(
                    request_id = %error_context.request_id,
                    email = %subscriber.email,
                    error = %e,
                    "Failed to send newsletter to confirmed subscriber"
                );
            }
        }
    }

    tracing::info!(
        request_id = %error_context.request_id,
        sent_count = sent_count,
        failed_count = failed_count,
        "Newsletter send to confirmed subscribers completed"
    );

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "message": "Newsletter sent to confirmed subscribers",
        "sent_count": sent_count,
        "failed_count": failed_count
    })))
}

/// Fetch all subscribers from database
async fn get_all_subscribers(
    pool: &web::Data<PgPool>,
    context: &ErrorContext,
) -> Result<Vec<SubscriberData>, AppError> {
    let subscribers = sqlx::query_as::<_, SubscriberData>(
        "SELECT id, email, name, status FROM subscriptions"
    )
    .fetch_all(pool.get_ref())
    .await
    .map_err(|e| {
        let error = AppError::Database(DatabaseError::UnexpectedError(
            format!("Failed to fetch subscribers: {}", e)
        ));
        context.log_error(&error);

        let request_metadata = RequestMetadata::new(
            context.request_id.clone(),
            "POST".to_string(),
            "/newsletters/send-all".to_string(),
        );

        let error_message = format!("Failed to fetch subscribers: {}", e);
        let failed_request = FailedRequest::new(
            request_metadata,
            "DatabaseError".to_string(),
            error_message.clone(),
            "DATABASE_ERROR".to_string(),
            500,
        )
        .with_retryable(true);

        RequestFailureLogger::log_failed_request(&failed_request);

        let audit_log = AuditLog::new(
            "FETCH_SUBSCRIBERS".to_string(),
            "newsletter".to_string(),
            "FAILURE".to_string(),
            error_message,
        );
        RequestFailureLogger::log_audit(&audit_log);

        error
    })?;

    Ok(subscribers)
}

/// Fetch only confirmed subscribers from database
async fn get_confirmed_subscribers(
    pool: &web::Data<PgPool>,
    context: &ErrorContext,
) -> Result<Vec<SubscriberData>, AppError> {
    let subscribers = sqlx::query_as::<_, SubscriberData>(
        "SELECT id, email, name, status FROM subscriptions WHERE status = 'confirmed'"
    )
    .fetch_all(pool.get_ref())
    .await
    .map_err(|e| {
        let error = AppError::Database(DatabaseError::UnexpectedError(
            format!("Failed to fetch confirmed subscribers: {}", e)
        ));
        context.log_error(&error);

        let request_metadata = RequestMetadata::new(
            context.request_id.clone(),
            "POST".to_string(),
            "/newsletters/send-confirmed".to_string(),
        );

        let error_message = format!("Failed to fetch confirmed subscribers: {}", e);
        let failed_request = FailedRequest::new(
            request_metadata,
            "DatabaseError".to_string(),
            error_message.clone(),
            "DATABASE_ERROR".to_string(),
            500,
        )
        .with_retryable(true);

        RequestFailureLogger::log_failed_request(&failed_request);

        let audit_log = AuditLog::new(
            "FETCH_CONFIRMED_SUBSCRIBERS".to_string(),
            "newsletter".to_string(),
            "FAILURE".to_string(),
            error_message,
        );
        RequestFailureLogger::log_audit(&audit_log);

        error
    })?;

    Ok(subscribers)
}
