use actix_web::{web, App, HttpResponse, HttpServer, Responder};
use serde::Deserialize;
use std::net::TcpListener;
use actix_web::dev::Server;

#[derive(Deserialize)]
struct FormData {
    name: Option<String>,
    email: Option<String>,
}

async fn greet(req: actix_web::HttpRequest) -> impl Responder {
    let name = req.match_info().get("name").unwrap_or("World");
    format!("Hello {}!", name)
}

async fn health_check() -> impl Responder {
    "OK"
}

/// 이메일 검증 헬퍼 함수
fn is_valid_email(email: &str) -> bool {
    let trimmed = email.trim();
    !trimmed.is_empty() && trimmed.contains('@') && trimmed.len() > 5
}

async fn subscribe(form: web::Form<FormData>) -> HttpResponse {
    // 이름 검증: 공백 제거 후 비어있지 않은지 확인
    let name_valid = form.name
        .as_ref()
        .map(|n| !n.trim().is_empty())
        .unwrap_or(false);

    // 이메일 검증: 형식 확인
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

pub fn startup(listener: TcpListener) -> Result<Server, std::io::Error> {
    let server = HttpServer::new(|| {
        App::new()
            .route("/health_check", web::get().to(health_check))
            .route("/subscriptions", web::post().to(subscribe))
            .route("/", web::get().to(greet))
            .route("/{name}", web::get().to(greet))
    })
    .listen(listener)?
    .run();

    Ok(server)
}

pub async fn run() -> std::io::Result<Server> {
    let listener = TcpListener::bind("127.0.0.1:8080")?;
    startup(listener)
}

