use actix_web::{web, HttpResponse};
use chrono::Utc;
use serde::Deserialize;
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Deserialize)]
pub struct FormData {
    name: Option<String>,
    email: Option<String>,
}

/// Email validation helper function
fn is_valid_email(email: &str) -> bool {
    let trimmed = email.trim();
    !trimmed.is_empty() && trimmed.contains('@') && trimmed.len() > 5
}

pub async fn subscribe(
    form: web::Form<FormData>,
    pool: web::Data<PgPool>,
) -> HttpResponse {
    // Name validation: check if not empty after trimming
    let name_valid = form.name
        .as_ref()
        .map(|n| !n.trim().is_empty())
        .unwrap_or(false);

    // Email validation: format check
    let email_valid = form.email
        .as_ref()
        .map(|e| is_valid_email(e))
        .unwrap_or(false);

    if !name_valid || !email_valid {
        tracing::warn!(
            name_valid = name_valid,
            email_valid = email_valid,
            "Invalid subscription request received"
        );
        return HttpResponse::BadRequest().finish();
    }

    let name = form.name.as_ref().unwrap().trim();
    let email = form.email.as_ref().unwrap().trim();

    tracing::info!(
        email = %email,
        name = %name,
        "Processing new subscription"
    );

    let subscriber_id = Uuid::new_v4();

    match sqlx::query(
        "INSERT INTO subscriptions (id, email, name, subscribed_at) VALUES ($1, $2, $3, $4)"
    )
    .bind(subscriber_id)
    .bind(email)
    .bind(name)
    .bind(Utc::now())
    .execute(pool.get_ref())
    .await
    {
        Ok(_) => {
            tracing::info!(
                subscriber_id = %subscriber_id,
                email = %email,
                "New subscriber saved successfully"
            );
            HttpResponse::Ok().finish()
        }
        Err(e) => {
            tracing::error!(
                subscriber_id = %subscriber_id,
                email = %email,
                error = %e,
                "Failed to save subscriber to database"
            );
            HttpResponse::InternalServerError().finish()
        }
    }
}
