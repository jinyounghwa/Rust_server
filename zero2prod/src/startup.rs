use actix_web::{web, App, HttpServer};
use sqlx::PgConnection;
use std::net::TcpListener;
use actix_web::dev::Server;
use crate::routes::{health_check, subscribe};

pub fn run(listener: TcpListener, connection:PgConnection) -> Result<Server, std::io::Error> {
    let connection = web::Data::new(connection);
    let server = HttpServer::new(move || {
        App::new()
            .app_data(connection.clone())
            .route("/health_check", web::get().to(health_check))
            .route("/subscriptions", web::post().to(subscribe))
    })
    .listen(listener)?
    .run();
    Ok(server)
}
