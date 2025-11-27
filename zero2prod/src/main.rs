use std::net::TcpListener;
use zero2prod::configuration::get_configuration;
use zero2prod::startup::run;
use zero2prod::telemetry::init_telemetry;
use sqlx::postgres::PgPoolOptions;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    // 구조화된 로깅 초기화
    init_telemetry();

    tracing::info!("Starting application");

    // 설정 로드
    let configuration = match get_configuration() {
        Ok(config) => {
            tracing::info!("Configuration loaded successfully");
            config
        }
        Err(e) => {
            tracing::error!("Failed to read configuration: {}", e);
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "Configuration error"
            ));
        }
    };

    // 데이터베이스 연결 풀 생성
    let connection_string = configuration.database.connection_string();
    tracing::info!("Attempting to connect to database");

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&connection_string)
        .await
        .map_err(|e| {
            tracing::error!("Failed to create connection pool: {}", e);
            std::io::Error::new(
                std::io::ErrorKind::ConnectionRefused,
                "Database connection error"
            )
        })?;

    tracing::info!("Database connection pool created successfully");

    // 서버 주소 설정
    let address = format!("127.0.0.1:{}", configuration.application.port);
    tracing::info!("Binding server to address: {}", address);

    let listener = TcpListener::bind(&address)?;
    tracing::info!("Server listening on: {}", address);

    // JWT 설정 저장
    let jwt_config = configuration.jwt.clone();

    // 서버 실행
    let server = run(listener, pool, jwt_config)?;
    tracing::info!("Server started successfully");

    let _ = server.await;

    Ok(())
}   
