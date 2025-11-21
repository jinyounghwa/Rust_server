use actix_web::{App, HttpServer, middleware::Logger, web};
use sqlx::PgPool;
use std::net::TcpListener;
use actix_web::dev::Server;
use crate::routes::{health_check, subscribe, confirm_subscription};
use crate::logger::LoggerMiddleware;

pub fn run(listener: TcpListener, connection: PgPool) -> Result<Server, std::io::Error> {
    let connection = web::Data::new(connection);

    let server = HttpServer::new(move || {
        App::new()
            // Security middleware stack
            .wrap(Logger::default())                   // Logging
            .wrap(LoggerMiddleware)                    // Custom logging
            .app_data(connection.clone())
            .route("/health_check", web::get().to(health_check))
            .route("/subscriptions", web::post().to(subscribe))
            .route("/subscriptions/confirm", web::get().to(confirm_subscription))
    })
    .listen(listener)?
    .run();
    Ok(server)
}
