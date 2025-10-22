use actix_web::{web, App, HttpServer, Responder};
use std::net::TcpListener;

async fn greet(req: actix_web::HttpRequest) -> impl Responder {
    let name = req.match_info().get("name").unwrap_or("World");
    format!("Hello {}!", name)
}

async fn health_check() -> impl Responder {
    "OK"
}

pub fn startup(listener: TcpListener) -> Result<actix_web::dev::Server, std::io::Error> {
    let server = HttpServer::new(|| {
        App::new()
            .route("/health_check", web::get().to(health_check))
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