# 설정(Configuration) 가이드

## 개요
이 문서는 `zero2prod` 프로젝트의 설정 시스템을 설명합니다. YAML 파일을 사용하여 애플리케이션과 데이터베이스 설정을 관리합니다.

## 수정된 사항

### 1. **Cargo.toml 수정**
**문제**: `config` 크레이트가 의존성 섹션에 중복되어 있었고, 구조가 잘못되어 있었습니다.

**해결**:
```toml
[dependencies]
actix-web = "4"
tokio = {version = "1", features = ["macros", "rt-multi-thread"]}
serde = {version = "1", features = ["derive"]}
sqlx = {version = "0.6", features = ["postgres", "runtime-tokio-native-tls"]}
config = "0.13"  # ← 올바른 위치에 추가
```

중복된 의존성 선언을 제거하고 `config` 크레이트를 `[dependencies]` 섹션에 올바르게 배치했습니다.

### 2. **configration.rs 수정**
**문제들**:
- `config` 모듈을 import하지 않았음
- 설정 구조가 YAML 파일과 일치하지 않음
- `application_port`가 최상위 구조에 있어 유연성이 부족함
- Clone trait가 없어 재사용이 어려움

**해결**:

```rust
use config::ConfigError;

#[derive(serde::Deserialize, Clone)]
pub struct Settings {
    pub database: DatabaseSettings,
    pub application: ApplicationSettings,
}

#[derive(serde::Deserialize, Clone)]
pub struct ApplicationSettings {
    pub port: u16,
}

#[derive(serde::Deserialize, Clone)]
pub struct DatabaseSettings {
    pub username: String,
    pub password: String,
    pub port: u16,
    pub host: String,
    pub database_name: String,
}

pub fn get_configuration() -> Result<Settings, ConfigError> {
    let settings = config::Config::builder()
        .add_source(config::File::with_name("configuration").required(false))
        .build()?;
    settings.try_deserialize::<Settings>()
}
```

**개선 사항**:
- `use config::ConfigError` import 추가
- `ApplicationSettings` 구조체 신규 생성으로 설정 조직화 개선
- `Clone` trait 추가로 설정 공유 가능
- `config::File::with_name()` 사용으로 확장자 자동 감지 (`.yaml`, `.toml` 등)
- `required(false)` 옵션으로 기본값 처리 가능

### 3. **configuration.yaml 파일 생성**
새로운 설정 파일을 생성했습니다:

```yaml
application:
  port: 8000

database:
  username: postgres
  password: password
  port: 5432
  host: localhost
  database_name: zero2prod
```

**YAML 구조 설명**:
- **application**: 애플리케이션 관련 설정
  - `port`: 서버가 실행될 포트 (기본값: 8000)
- **database**: PostgreSQL 데이터베이스 설정
  - `username`: DB 사용자명
  - `password`: DB 비밀번호
  - `port`: PostgreSQL 포트 (기본값: 5432)
  - `host`: 데이터베이스 호스트 주소
  - `database_name`: 사용할 데이터베이스명

## 사용법

### 1. 설정 로드하기
```rust
use configration::get_configuration;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let config = get_configuration()
        .expect("Failed to read configuration");

    let address = format!("127.0.0.1:{}", config.application.port);
    println!("Server running at {}", address);
    // ...
}
```

### 2. 환경별 설정
다양한 환경에 대응하려면 다음과 같이 확장할 수 있습니다:

```rust
pub fn get_configuration() -> Result<Settings, ConfigError> {
    let env = std::env::var("APP_ENV").unwrap_or_else(|_| "development".to_string());

    let settings = config::Config::builder()
        .add_source(config::File::with_name("configuration").required(false))
        .add_source(config::File::with_name(&format!("configuration.{}", env)).required(false))
        .build()?;
    settings.try_deserialize::<Settings>()
}
```

파일 구조:
- `configuration.yaml` - 기본 설정
- `configuration.development.yaml` - 개발 환경
- `configuration.production.yaml` - 프로덕션 환경

### 3. 환경 변수로 오버라이드
```rust
pub fn get_configuration() -> Result<Settings, ConfigError> {
    let settings = config::Config::builder()
        .add_source(config::File::with_name("configuration").required(false))
        .add_source(config::Environment::with_prefix("APP").try_parsing(true).separator("__"))
        .build()?;
    settings.try_deserialize::<Settings>()
}
```

이 경우 `APP_DATABASE__USERNAME=myuser`와 같이 환경 변수로 설정을 오버라이드할 수 있습니다.

## 에러 처리

설정 로드 실패 시 적절한 에러 처리:

```rust
match get_configuration() {
    Ok(config) => {
        println!("Configuration loaded successfully");
        // 애플리케이션 시작
    },
    Err(e) => {
        eprintln!("Failed to load configuration: {}", e);
        std::process::exit(1);
    }
}
```

## 의존성
- **config 0.13**: YAML, TOML, JSON 등 다양한 포맷 지원
- **serde**: 설정 구조체 직렬화/역직렬화

## 트러블슈팅

### configuration.yaml를 찾을 수 없음
- 파일이 프로젝트 루트 디렉토리에 있는지 확인
- `cargo run`을 실행하는 디렉토리에서 파일 경로 확인

### 설정값이 로드되지 않음
- YAML 문법 확인 (들여쓰기, 콜론 등)
- 데이터 타입이 Rust 구조체와 일치하는지 확인
- `required(false)` 옵션으로 설정 파일 로드 실패 무시 가능

## 보안 주의사항
- 프로덕션 환경에서는 민감한 정보(비밀번호, API 키 등)를 YAML 파일에 저장하지 마세요
- `.gitignore`에 설정 파일 추가: `configuration.*.yaml`
- 환경 변수나 시크릿 관리 도구 사용 권장
