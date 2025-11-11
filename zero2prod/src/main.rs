use std::net::TcpListener;
use zero2prod::configuration::get_configuration;
use zero2prod::startup::run;
use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    // 환경 로거 초기화
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));

    let configuration = get_configuration().expect("Failed to read configuration");
    let connection_pool: PgPool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&configuration.database.connection_string())
        .await
        .expect("Failed to create connection pool");
    let address = format!("127.0.0.1:{}", configuration.application.port);
    let listener = TcpListener::bind(address)?;
    let server = run(listener, connection_pool).expect("Failed to create server");
    let _ = server.await;
    Ok(())
}