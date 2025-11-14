# 구조화된 로깅 (Structured Logging) 가이드

## 목차
1. [개요](#개요)
2. [아키텍처](#아키텍처)
3. [사용 방법](#사용-방법)
4. [로그 포맷](#로그-포맷)
5. [로그 레벨 제어](#로그-레벨-제어)
6. [모범 사례](#모범-사례)
7. [트러블슈팅](#트러블슈팅)

---

## 개요

### 구조화된 로깅이란?

구조화된 로깅은 텍스트 기반의 로그 메시지 대신 **구조화된 데이터(JSON)**로 로그를 기록하는 방식입니다.

**기존 방식:**
```
2025-11-14 10:00:00 INFO - Request completed: POST /subscriptions - Status: 200 (45ms)
```

**구조화된 로깅:**
```json
{
  "timestamp": "2025-11-14T10:00:00.000Z",
  "level": "INFO",
  "message": "HTTP request completed",
  "method": "POST",
  "path": "/subscriptions",
  "status": 200,
  "elapsed_ms": 45
}
```

### 구조화된 로깅의 이점

| 이점 | 설명 |
|------|------|
| **검색 용이성** | 각 필드가 독립적이므로 로그 검색이 정확하고 빠름 |
| **분석 자동화** | JSON 형식이므로 프로그래밍 방식으로 쉽게 분석 가능 |
| **상관관계 추적** | 특정 요청이나 사용자를 따라 로그 체인 구성 가능 |
| **모니터링 통합** | ELK, Splunk 등의 로그 집계 도구와 쉽게 연동 |
| **일관성** | 구조화된 형식으로 모든 로그가 일관성 있게 기록됨 |

---

## 아키텍처

### 사용된 라이브러리

```toml
[dependencies]
tracing = "0.1"                    # 구조화된 이벤트 로깅 라이브러리
tracing-subscriber = {
    version = "0.3",
    features = ["json", "env-filter", "fmt"]
}                                  # 로그 포매팅 및 필터링
serde_json = "1.0"                 # JSON 직렬화
```

### 핵심 구성

```
┌─────────────────┐
│   Application   │
└────────┬────────┘
         │ tracing::info!(), tracing::error!() 등으로 로그 발행
         │
┌────────▼────────────────────────────────────┐
│  Tracing Subscriber (텔레메트리 시스템)     │
├─────────────────────────────────────────────┤
│ ┌──────────────┐  ┌───────────────────┐   │
│ │ EnvFilter    │  │ JSON Formatter    │   │
│ │ (필터링)     │  │ (형식 변환)       │   │
│ └──────────────┘  └───────────────────┘   │
└────────┬─────────────────────────────────┘
         │ JSON 형식의 로그 출력
         ▼
      stdout
      (파일 또는 로그 수집 시스템으로 리다이렉트 가능)
```

### 모듈 구조

```
src/
├── telemetry.rs          # 로깅 초기화 로직
├── logger.rs             # HTTP 미들웨어 로깅
├── main.rs              # 애플리케이션 시작 로깅
└── routes/
    ├── subscriptions.rs  # 구독 기능 로깅
    └── health_check.rs   # 헬스 체크 로깅
```

---

## 사용 방법

### 1. 텔레메트리 초기화

애플리케이션 시작 시 `main.rs`에서 초기화해야 합니다:

```rust
use zero2prod::telemetry::init_telemetry;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    // 반드시 먼저 호출해야 함
    init_telemetry();

    tracing::info!("Starting application");
    // ... 나머지 로직
}
```

**`src/telemetry.rs`의 초기화 함수:**

```rust
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
```

### 2. 로그 작성

#### 기본 로깅

```rust
// 정보 레벨 로그
tracing::info!("Application started");

// 경고 레벨 로그
tracing::warn!("Deprecated API endpoint accessed");

// 에러 레벨 로그
tracing::error!("Database connection failed");

// 디버그 레벨 로그
tracing::debug!("Processing request");

// 추적 레벨 로그 (가장 상세함)
tracing::trace!("Entering function with parameters: {:?}", params);
```

#### 구조화된 필드를 포함한 로깅

구조화된 로깅의 핵심은 **명명된 필드(named fields)**입니다:

```rust
let subscriber_id = uuid::Uuid::new_v4();
let email = "user@example.com";

// 구조화된 필드 추가
tracing::info!(
    subscriber_id = %subscriber_id,
    email = %email,
    "New subscriber saved successfully"
);
```

**출력 (JSON 형식):**
```json
{
  "timestamp": "2025-11-14T10:00:00.000Z",
  "level": "INFO",
  "message": "New subscriber saved successfully",
  "subscriber_id": "550e8400-e29b-41d4-a716-446655440000",
  "email": "user@example.com"
}
```

#### 필드 포맷터

| 포맷터 | 설명 | 예 |
|--------|------|-----|
| `field = value` | Display 트레이트로 값 표시 | `status = 200` |
| `field = %value` | Display 포맷터 (명시적) | `name = %name` |
| `field = ?value` | Debug 포맷터 | `error = ?err` |
| `field = %?value` | Debug + Display 혼합 | `data = %?data` |

#### 실제 예제

**HTTP 요청 로깅 (`src/logger.rs`):**

```rust
fn call(&self, req: ServiceRequest) -> Self::Future {
    let start_time = Instant::now();
    let method = req.method().to_string();
    let path = req.path().to_string();

    // 요청 시작 로그
    tracing::info!(
        method = %method,
        path = %path,
        "HTTP request received"
    );

    let service = self.service.clone();

    Box::pin(async move {
        let res = service.call(req).await?;

        let elapsed = start_time.elapsed();
        let status = res.status().as_u16();
        let elapsed_ms = elapsed.as_millis();

        // 요청 완료 로그
        tracing::info!(
            method = %method,
            path = %path,
            status = status,
            elapsed_ms = elapsed_ms,
            "HTTP request completed"
        );

        Ok(res)
    })
}
```

**구독 처리 로깅 (`src/routes/subscriptions.rs`):**

```rust
pub async fn subscribe(
    form: web::Form<FormData>,
    pool: web::Data<PgPool>,
) -> HttpResponse {
    // ... 유효성 검증 ...

    if !name_valid || !email_valid {
        tracing::warn!(
            name_valid = name_valid,
            email_valid = email_valid,
            "Invalid subscription request received"
        );
        return HttpResponse::BadRequest().finish();
    }

    let name = form.name.as_ref().unwrap().trim();
    let email = form.email.as_ref().unwrap().trim();
    let subscriber_id = Uuid::new_v4();

    tracing::info!(
        email = %email,
        name = %name,
        "Processing new subscription"
    );

    match sqlx::query(
        "INSERT INTO subscriptions (id, email, name, subscribed_at) VALUES ($1, $2, $3, $4)"
    )
    .bind(subscriber_id)
    .bind(email)
    .bind(name)
    .bind(Utc::now())
    .execute(pool.get_ref())
    .await
    {
        Ok(_) => {
            tracing::info!(
                subscriber_id = %subscriber_id,
                email = %email,
                "New subscriber saved successfully"
            );
            HttpResponse::Ok().finish()
        }
        Err(e) => {
            tracing::error!(
                subscriber_id = %subscriber_id,
                email = %email,
                error = %e,
                "Failed to save subscriber to database"
            );
            HttpResponse::InternalServerError().finish()
        }
    }
}
```

---

## 로그 포맷

### JSON 로그 구조

모든 로그는 다음과 같은 구조의 JSON으로 출력됩니다:

```json
{
  "timestamp": "2025-11-14T10:00:00.000Z",
  "level": "INFO",
  "fields": {
    "message": "HTTP request completed",
    "method": "POST",
    "path": "/subscriptions",
    "status": 200,
    "elapsed_ms": 45
  },
  "target": "zero2prod",
  "span": {
    "name": "subscribe"
  }
}
```

### 필드 설명

| 필드 | 설명 |
|------|------|
| `timestamp` | 로그 발생 시간 (ISO 8601 형식) |
| `level` | 로그 레벨 (TRACE, DEBUG, INFO, WARN, ERROR) |
| `message` | 로그 메시지 |
| `fields` | 구조화된 필드 객체 |
| `target` | 로그 출처 (모듈명 또는 크레이트명) |
| `span` | 현재 실행 span 정보 |

---

## 로그 레벨 제어

### 환경 변수를 통한 로그 레벨 설정

`RUST_LOG` 환경 변수로 로그 레벨을 제어합니다:

#### 전체 애플리케이션 로그 레벨 설정

```bash
# 모든 로그를 INFO 레벨로 표시
RUST_LOG=info cargo run

# 모든 로그를 DEBUG 레벨로 표시
RUST_LOG=debug cargo run

# 모든 로그를 ERROR 레벨로 표시 (가장 적음)
RUST_LOG=error cargo run
```

#### 모듈별 로그 레벨 설정

```bash
# zero2prod 모듈은 debug, 다른 것은 info
RUST_LOG=zero2prod=debug,info cargo run

# 특정 모듈만 추적
RUST_LOG=zero2prod::routes::subscriptions=debug cargo run

# 여러 모듈 지정
RUST_LOG=zero2prod=debug,actix_web=info cargo run
```

#### 로그 레벨 계층

```
TRACE (5)  - 가장 상세, 모든 정보 (프로그래밍 흐름 추적)
  ↓
DEBUG (4)  - 개발 중 유용한 정보
  ↓
INFO  (3)  - 일반적인 정보 메시지 (기본값)
  ↓
WARN  (2)  - 경고 메시지 (문제 가능성)
  ↓
ERROR (1)  - 에러 메시지 (심각한 문제)
  ↓
OFF   (0)  - 로깅 비활성화
```

### 기본 로그 레벨

코드에서 설정한 기본값 (`src/telemetry.rs`):

```rust
let env_filter = tracing_subscriber::EnvFilter::try_from_default_env()
    .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info"));
    // RUST_LOG 환경 변수가 없으면 "info" 레벨이 기본값
```

---

## 모범 사례

### 1. 언제 어떤 레벨을 사용할까?

**ERROR:**
```rust
// 데이터베이스 연결 실패, API 호출 실패 등 심각한 문제
tracing::error!(
    database_url = %db_url,
    error = %e,
    "Failed to connect to database"
);
```

**WARN:**
```rust
// 잘못된 입력, 재시도 가능한 오류
tracing::warn!(
    name_valid = false,
    email_valid = false,
    "Invalid subscription request received"
);
```

**INFO:**
```rust
// 중요한 비즈니스 이벤트
tracing::info!(
    subscriber_id = %subscriber_id,
    email = %email,
    "New subscriber saved successfully"
);
```

**DEBUG:**
```rust
// 개발 중 유용한 디버깅 정보
tracing::debug!(
    query = "SELECT * FROM users",
    "Executing database query"
);
```

### 2. 구조화된 필드 명명 규칙

```rust
// ✅ 좋은 예
tracing::info!(
    user_id = %user_id,
    email = %email,
    request_duration_ms = duration_ms,
    "User login successful"
);

// ❌ 나쁜 예 - 필드명이 모호함
tracing::info!(
    id = %some_id,
    data = %some_data,
    "Operation completed"
);
```

### 3. 민감한 정보 처리

```rust
// ❌ 위험: 비밀번호가 로그에 기록됨
tracing::info!(
    username = %username,
    password = %password,
    "User login attempted"
);

// ✅ 안전: 필요한 정보만 로그
tracing::info!(
    username = %username,
    "User login attempted"
);

// ✅ 더 나은 예: 해시 사용
tracing::info!(
    username = %username,
    password_hash = %hash(&password),
    "User login attempted"
);
```

### 4. 오류 상황에서의 상세 로깅

```rust
// ✅ 문제 진단에 필요한 정보 포함
tracing::error!(
    subscriber_id = %subscriber_id,
    email = %email,
    error_message = %e.to_string(),
    error_details = %format!("{:?}", e),
    "Failed to save subscriber to database"
);
```

### 5. 요청 추적을 위한 상관 ID

```rust
// UUID를 사용하여 관련 로그들을 연결
let correlation_id = uuid::Uuid::new_v4();

tracing::info!(
    correlation_id = %correlation_id,
    "Request started"
);

// ... 작업 수행 ...

tracing::info!(
    correlation_id = %correlation_id,
    "Request completed"
);
```

---

## 트러블슈팅

### 문제 1: 로그가 나타나지 않음

**원인:** 로그 레벨이 높게 설정됨

```bash
# 해결책: 더 낮은 레벨로 설정
RUST_LOG=debug cargo run
```

### 문제 2: `init_telemetry()` 호출 오류

**원인:** 텔레메트리가 이미 초기화됨 (메인 함수에서 2번 호출)

```rust
// ❌ 문제: 두 번 호출됨
#[tokio::main]
async fn main() {
    init_telemetry();

    some_function();  // 여기서도 init_telemetry() 호출
}

// ✅ 해결책: 메인 함수에서만 호출
#[tokio::main]
async fn main() {
    init_telemetry();  // 메인에서만 호출
    // ...
}
```

### 문제 3: 로그 출력 형식이 JSON이 아님

**원인:** 텔레메트리가 초기화되지 않음

```rust
// ❌ 문제: init_telemetry() 호출 전에 로그 사용
#[tokio::main]
async fn main() {
    tracing::info!("Starting");  // 텔레메트리 미초기화
    init_telemetry();            // 너무 늦음
}

// ✅ 해결책: 초기화 먼저
#[tokio::main]
async fn main() {
    init_telemetry();            // 먼저 초기화
    tracing::info!("Starting");  // 이제 JSON으로 출력됨
}
```

### 문제 4: 성능 저하

**원인:** 로그 레벨이 TRACE로 설정됨 (매우 상세)

```bash
# ❌ 성능 저하
RUST_LOG=trace cargo run

# ✅ 개선: 필요한 모듈만 TRACE 사용
RUST_LOG=zero2prod=debug,info cargo run
```

### 문제 5: 로그 파일이 너무 커짐

**해결책:** 다음과 같이 로그를 외부 시스템으로 전송

```bash
# stdout을 파일로 리다이렉트하되, 순환 저장
RUST_LOG=info cargo run > app.log 2>&1 &

# 또는 다른 로그 집계 도구 사용 (ELK, Datadog 등)
```

---

## 로그 분석 예제

### jq를 사용한 JSON 로그 필터링

```bash
# 모든 로그 출력
RUST_LOG=info cargo run 2>&1 | jq .

# ERROR 레벨 로그만 필터링
RUST_LOG=info cargo run 2>&1 | jq 'select(.level == "ERROR")'

# 특정 경로의 로그만 필터링
RUST_LOG=info cargo run 2>&1 | jq 'select(.fields.path == "/subscriptions")'

# 느린 요청 찾기 (100ms 이상)
RUST_LOG=info cargo run 2>&1 | jq 'select(.fields.elapsed_ms > 100)'

# 특정 사용자 관련 모든 로그 추출
RUST_LOG=info cargo run 2>&1 | jq 'select(.fields.email == "user@example.com")'
```

### 로그 집계 도구 연동

**ELK Stack (Elasticsearch, Logstash, Kibana):**

```bash
# stdout을 Filebeat로 수집
RUST_LOG=info cargo run 2>&1 | nc localhost 5000
```

**Datadog:**

```bash
# Datadog 에이전트로 로그 전송
RUST_LOG=info cargo run 2>&1 | dd-agent-log-forwarder
```

---

## 참고 자료

### 공식 문서
- [Tracing Documentation](https://docs.rs/tracing/)
- [Tracing Subscriber Documentation](https://docs.rs/tracing-subscriber/)

### 추가 학습
- [Structured Logging 모범 사례](https://www.kartar.net/2015/12/structured-logging/)
- [JSON Logging 패턴](https://www.loggly.com/blog/json-logging-benefits-and-how-to-get-started/)

---

## 요약

| 항목 | 설명 |
|------|------|
| **라이브러리** | `tracing` + `tracing-subscriber` |
| **로그 포맷** | JSON |
| **초기화** | `init_telemetry()` - main에서 호출 |
| **로그 기록** | `tracing::info!()`, `tracing::error!()` 등 |
| **로그 레벨** | TRACE, DEBUG, INFO (기본), WARN, ERROR |
| **레벨 제어** | `RUST_LOG` 환경 변수 |
| **구조화 필드** | `field = %value` 형식 |
| **분석** | jq, ELK, Datadog 등과 연동 |

---

**마지막 수정:** 2025-11-14
**버전:** 1.0
