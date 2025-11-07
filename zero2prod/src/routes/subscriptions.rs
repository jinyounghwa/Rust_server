use actix_web::{web, HttpResponse};
use serde::Deserialize;

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

pub async fn subscribe(form: web::Form<FormData>) -> HttpResponse {
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

    if name_valid && email_valid {
        HttpResponse::Ok().finish()
    } else {
        HttpResponse::BadRequest().finish()
    }
}
