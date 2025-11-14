use actix_web::HttpResponse;

pub async fn health_check() -> HttpResponse {
    tracing::debug!("Health check endpoint called");
    HttpResponse::Ok().finish()
}
