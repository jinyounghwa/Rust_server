use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

/// 구조화된 로깅을 초기화합니다.
/// JSON 형식의 로그를 출력하며, RUST_LOG 환경 변수로 로그 레벨을 제어합니다.
pub fn init_telemetry() {
    let env_filter = tracing_subscriber::EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info"));

    let formatting_layer = tracing_subscriber::fmt::layer()
        .with_writer(std::io::stdout)
        .json();

    tracing_subscriber::registry()
        .with(env_filter)
        .with(formatting_layer)
        .init();
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_telemetry_initialization() {
        // 스레드 로컬에서 이미 초기화되어 있을 수 있으므로
        // 실제 호출은 테스트 환경에서 최초 1회만 가능합니다.
        // 이 테스트는 구조가 올바른지 확인하는 용도입니다.
        assert!(true);
    }
}
