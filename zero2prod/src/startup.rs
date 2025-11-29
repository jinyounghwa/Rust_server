use actix_web::{middleware::Logger, web, App, HttpServer};
use actix_files as fs;
use sqlx::PgPool;
use std::net::TcpListener;
use actix_web::dev::Server;

use crate::configuration::JwtSettings;
use crate::logger::LoggerMiddleware;
use crate::middleware::JwtMiddleware;
use crate::routes::{
    confirm_subscription, get_current_user, health_check, login, refresh, register, send_newsletter_to_all,
    send_newsletter_to_confirmed, subscribe,
};

pub fn run(
    listener: TcpListener,
    connection: PgPool,
    jwt_config: JwtSettings,
) -> Result<Server, std::io::Error> {
    let connection = web::Data::new(connection);
    let jwt_config_data = web::Data::new(jwt_config.clone());

    let server = HttpServer::new(move || {
        App::new()
            // Global middleware
            .wrap(Logger::default())      // Standard logging
            .wrap(LoggerMiddleware)       // Custom logging

            // Shared state
            .app_data(connection.clone())
            .app_data(jwt_config_data.clone())

            // Public routes (no authentication required)
            .route("/health_check", web::get().to(health_check))
            .route("/auth/register", web::post().to(register))
            .route("/auth/login", web::post().to(login))
            .route("/auth/refresh", web::post().to(refresh))

            // Protected routes (require JWT authentication)
            .service(
                web::scope("/api")
                    .wrap(JwtMiddleware::new(jwt_config.clone()))
                    .route("/me", web::get().to(get_current_user))
            )
            .route("/auth/me", web::get().to(get_current_user))
            .route("/subscriptions", web::post().to(subscribe))
            .route("/subscriptions/confirm", web::get().to(confirm_subscription))
            .route("/newsletters/send-all", web::post().to(send_newsletter_to_all))
            .route("/newsletters/send-confirmed", web::post().to(send_newsletter_to_confirmed))
            
            // Static file serving (must be last to not override API routes)
            .service(fs::Files::new("/", "./public").index_file("index.html"))
    })
    .listen(listener)?
    .run();

    Ok(server)
}
