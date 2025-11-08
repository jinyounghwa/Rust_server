use std::net::TcpListener;
use zero2prod::configuration::get_configuration;
use zero2prod::startup::run;
use sqlx::postgres::PgPoolOptions;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let configuration = get_configuration().expect("Failed to read configuration");
    let connection_pool = PgPoolOptions::new()
        .connect(&configuration.database.connection_string())
        .await
        .expect("Failed to create connection pool");
    let address = format!("127.0.0.1:{}", configuration.application.port);
    let listener = TcpListener::bind(address)?;
    let server = run(listener, connection_pool)?;
    let _ = server.await;
    Ok(())
}