use actix_web::{web, App, HttpResponse, HttpServer, Responder};
use serde::Deserialize;
use std::net::TcpListener;

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

async fn subscribe(form: web::Form<FormData>) -> HttpResponse {
    // Check if name is provided and not empty
    let name_valid = form.name
        .as_ref()
        .map(|n| !n.trim().is_empty())
        .unwrap_or(false);

    // Check if email is provided and not empty
    let email_valid = form.email
        .as_ref()
        .map(|e| !e.trim().is_empty())
        .unwrap_or(false);

    if name_valid && email_valid {
        HttpResponse::Ok().finish()
    } else {
        HttpResponse::BadRequest().finish()
    }
}

pub fn startup(listener: TcpListener) -> Result<actix_web::dev::Server, std::io::Error> {
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

pub async fn run() -> std::io::Result<()> {
    println!("Starting server on 127.0.0.1:8080");

    let listener = TcpListener::bind("127.0.0.1:8080")?;
    let server = startup(listener)?;
    server.await
}