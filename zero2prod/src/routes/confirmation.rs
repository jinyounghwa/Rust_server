use actix_web::{web, HttpResponse};
use serde::Deserialize;
use sqlx::PgPool;

#[derive(Deserialize)]
pub struct ConfirmationQuery {
    token: String,
}

pub async fn confirm_subscription(
    query: web::Query<ConfirmationQuery>,
    pool: web::Data<PgPool>,
) -> HttpResponse {
    let token = &query.token;

    // Check if token is valid and exists
    match get_subscriber_id_from_token(pool.get_ref(), token).await {
        Ok(Some(subscriber_id)) => {
            // Update subscription status to confirmed
            match update_subscription_status(pool.get_ref(), &subscriber_id, "confirmed").await {
                Ok(_) => {
                    tracing::info!(
                        subscriber_id = %subscriber_id,
                        "Subscription confirmed successfully"
                    );
                    HttpResponse::Ok().json(serde_json::json!({
                        "message": "Thank you for confirming your subscription!"
                    }))
                }
                Err(e) => {
                    tracing::error!(
                        subscriber_id = %subscriber_id,
                        error = %e,
                        "Failed to update subscription status"
                    );
                    HttpResponse::InternalServerError().json(serde_json::json!({
                        "error": "Failed to confirm subscription"
                    }))
                }
            }
        }
        Ok(None) => {
            tracing::warn!(
                token = %token,
                "Invalid or expired confirmation token"
            );
            HttpResponse::BadRequest().json(serde_json::json!({
                "error": "Invalid or expired confirmation token"
            }))
        }
        Err(e) => {
            tracing::error!(
                error = %e,
                "Database error while confirming subscription"
            );
            HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Failed to confirm subscription"
            }))
        }
    }
}

async fn get_subscriber_id_from_token(
    pool: &PgPool,
    token: &str,
) -> Result<Option<String>, sqlx::Error> {
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
    .await?;

    Ok(result.map(|(id,)| id))
}

async fn update_subscription_status(
    pool: &PgPool,
    subscriber_id: &str,
    status: &str,
) -> Result<(), sqlx::Error> {
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
    .await?;

    // Delete the token after successful confirmation
    sqlx::query(
        r#"
        DELETE FROM subscription_tokens
        WHERE subscriber_id = $1
        "#,
    )
    .bind(subscriber_id)
    .execute(pool)
    .await?;

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
