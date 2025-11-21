use actix_web::{web, HttpResponse};
use chrono::Utc;
use serde::Deserialize;
use sqlx::PgPool;
use uuid::Uuid;
use crate::validators::{is_valid_email, is_valid_name};
use crate::email_client::EmailClient;
use crate::confirmation_token::ConfirmationToken;

#[derive(Deserialize)]
pub struct FormData {
    name: Option<String>,
    email: Option<String>,
}

pub async fn subscribe(
    form: web::Form<FormData>,
    pool: web::Data<PgPool>,
    email_client: web::Data<EmailClient>,
) -> HttpResponse {
    // Name validation with comprehensive security checks
    let name = match form.name.as_ref() {
        Some(n) => match is_valid_name(n) {
            Ok(validated) => validated,
            Err(e) => {
                tracing::warn!(
                    error = %e,
                    "Invalid name received in subscription request"
                );
                return HttpResponse::BadRequest().finish();
            }
        },
        None => {
            tracing::warn!("Missing name in subscription request");
            return HttpResponse::BadRequest().finish();
        }
    };

    // Email validation with comprehensive security checks
    let email = match form.email.as_ref() {
        Some(e) => match is_valid_email(e) {
            Ok(validated) => validated,
            Err(err) => {
                tracing::warn!(
                    error = %err,
                    "Invalid email received in subscription request"
                );
                return HttpResponse::BadRequest().finish();
            }
        },
        None => {
            tracing::warn!("Missing email in subscription request");
            return HttpResponse::BadRequest().finish();
        }
    };

    tracing::info!(
        "Processing new subscription (sensitive data redacted)"
    );

    let subscriber_id = Uuid::new_v4();

    match sqlx::query(
        "INSERT INTO subscriptions (id, email, name, subscribed_at, status) VALUES ($1, $2, $3, $4, $5)"
    )
    .bind(subscriber_id)
    .bind(&email)
    .bind(&name)
    .bind(Utc::now())
    .bind("pending")
    .execute(pool.get_ref())
    .await
    {
        Ok(_) => {
            tracing::info!(
                subscriber_id = %subscriber_id,
                "New subscriber saved successfully"
            );

            // Generate confirmation token
            let confirmation_token = ConfirmationToken::new(subscriber_id);

            // Save token to database
            if let Err(e) = sqlx::query(
                r#"
                INSERT INTO subscription_tokens
                (subscription_token, subscriber_id, created_at, expires_at)
                VALUES ($1, $2, $3, $4)
                "#
            )
            .bind(confirmation_token.token())
            .bind(subscriber_id.to_string())
            .bind(confirmation_token.created_at())
            .bind(confirmation_token.expires_at())
            .execute(pool.get_ref())
            .await
            {
                tracing::error!(
                    subscriber_id = %subscriber_id,
                    error = %e,
                    "Failed to save confirmation token"
                );
                return HttpResponse::InternalServerError().finish();
            }

            // Send confirmation email
            let confirmation_link = format!(
                "http://localhost:8000/subscriptions/confirm?token={}",
                confirmation_token.token()
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

            if let Err(e) = send_confirmation_email(
                email_client.get_ref(),
                &email,
                &html_content,
            )
            .await
            {
                tracing::error!(
                    subscriber_id = %subscriber_id,
                    error = %e,
                    "Failed to send confirmation email"
                );
                return HttpResponse::InternalServerError().finish();
            }

            HttpResponse::Ok().finish()
        }
        Err(e) => {
            // Handle specific database errors
            let error_message = e.to_string();

            // Check for unique constraint violation (duplicate email)
            if error_message.contains("duplicate key") || error_message.contains("unique") {
                tracing::warn!(
                    subscriber_id = %subscriber_id,
                    "Duplicate email subscription attempt"
                );
                return HttpResponse::Conflict().finish();
            }

            tracing::error!(
                subscriber_id = %subscriber_id,
                error = %e,
                "Failed to save subscriber to database"
            );
            HttpResponse::InternalServerError().finish()
        }
    }
}

async fn send_confirmation_email(
    email_client: &EmailClient,
    recipient_email: &str,
    html_content: &str,
) -> Result<(), String> {
    email_client
        .send_email(
            recipient_email,
            "Please confirm your subscription",
            html_content,
        )
        .await
}
