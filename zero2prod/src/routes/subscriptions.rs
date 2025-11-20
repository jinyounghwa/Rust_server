use actix_web::{web, HttpResponse};
use chrono::Utc;
use serde::Deserialize;
use sqlx::PgPool;
use uuid::Uuid;
use crate::validators::{is_valid_email, is_valid_name};

#[derive(Deserialize)]
pub struct FormData {
    name: Option<String>,
    email: Option<String>,
}

pub async fn subscribe(
    form: web::Form<FormData>,
    pool: web::Data<PgPool>,
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
        "INSERT INTO subscriptions (id, email, name, subscribed_at) VALUES ($1, $2, $3, $4)"
    )
    .bind(subscriber_id)
    .bind(&email)
    .bind(&name)
    .bind(Utc::now())
    .execute(pool.get_ref())
    .await
    {
        Ok(_) => {
            tracing::info!(
                subscriber_id = %subscriber_id,
                "New subscriber saved successfully"
            );
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
