use actix_web::{web, HttpResponse};
use serde::Deserialize;
use sqlx::PgPool;
use crate::error::{AppError, DatabaseError, ErrorContext};

#[derive(Deserialize)]
pub struct ConfirmationQuery {
    token: String,
}

pub async fn confirm_subscription(
    query: web::Query<ConfirmationQuery>,
    pool: web::Data<PgPool>,
) -> Result<HttpResponse, AppError> {
    let error_context = ErrorContext::new("subscription_confirmation");
    let token = &query.token;

    tracing::info!(
        request_id = %error_context.request_id,
        token = %token,
        "Processing subscription confirmation"
    );

    // Get subscriber ID from token
    let subscriber_id = get_subscriber_id_from_token(pool.get_ref(), token, &error_context).await?;

    // Update subscription status to confirmed
    update_subscription_status(pool.get_ref(), &subscriber_id, "confirmed", &error_context).await?;

    tracing::info!(
        request_id = %error_context.request_id,
        subscriber_id = %subscriber_id,
        "Subscription confirmed successfully"
    );

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "message": "Thank you for confirming your subscription!",
        "request_id": error_context.request_id
    })))
}

async fn get_subscriber_id_from_token(
    pool: &PgPool,
    token: &str,
    context: &ErrorContext,
) -> Result<String, AppError> {
    let result = sqlx::query_as::<_, (String,)>(
        r#"
        SELECT subscriber_id
        FROM subscription_tokens
        WHERE subscription_token = $1
        AND expires_at > NOW()
        "#,
    )
    .bind(token)
    .fetch_optional(pool)
    .await
    .map_err(|e| {
        let error = AppError::from(e);
        context.log_error(&error);
        error
    })?;

    result.map(|(id,)| id).ok_or_else(|| {
        let error = AppError::Database(DatabaseError::NotFound(
            "Invalid or expired confirmation token".to_string()
        ));
        tracing::warn!(
            request_id = %context.request_id,
            "Invalid or expired confirmation token"
        );
        error
    })
}

async fn update_subscription_status(
    pool: &PgPool,
    subscriber_id: &str,
    status: &str,
    context: &ErrorContext,
) -> Result<(), AppError> {
    sqlx::query(
        r#"
        UPDATE subscriptions
        SET status = $1
        WHERE id = $2
        "#,
    )
    .bind(status)
    .bind(subscriber_id)
    .execute(pool)
    .await
    .map_err(|e| {
        let error = AppError::from(e);
        context.log_error(&error);
        error
    })?;

    // Delete the token after successful confirmation
    sqlx::query(
        r#"
        DELETE FROM subscription_tokens
        WHERE subscriber_id = $1
        "#,
    )
    .bind(subscriber_id)
    .execute(pool)
    .await
    .map_err(|e| {
        let error = AppError::from(e);
        context.log_error(&error);
        error
    })?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_confirmation_query_deserialization() {
        let json = r#"{"token": "test-token-123"}"#;
        let query: ConfirmationQuery = serde_json::from_str(json).unwrap();
        assert_eq!(query.token, "test-token-123");
    }
}
