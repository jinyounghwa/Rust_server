use actix_web::{App, HttpServer, middleware::Logger, web };
use sqlx::PgPool;
use std::net::TcpListener;
use actix_web::dev::Server;
use crate::routes::{health_check, subscribe};
use crate::logger::LoggerMiddleware;

pub fn run(listener: TcpListener, connection: PgPool) -> Result<Server, std::io::Error> {
    let connection = web::Data::new(connection);
    let server = HttpServer::new(move || {
        App::new()
            .wrap(Logger::default())
            .wrap(LoggerMiddleware)
            .app_data(connection.clone())
            .route("/health_check", web::get().to(health_check))
            .route("/subscriptions", web::post().to(subscribe))
    })
    .listen(listener)?
    .run();
    Ok(server)
}
