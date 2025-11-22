# 포괄적 오류 처리 시스템 (Comprehensive Error Handling System)

## 개요 (Overview)

이 문서는 Rust 웹 애플리케이션의 5가지 오류 처리 패턴을 설명합니다:

1. **제어 흐름에 대한 오류 처리 (Control Flow Error Handling)**
2. **운영자를 위한 오류 처리 (Operator/System Error Handling)**
3. **Error 트레이트 구현 (Error Trait Implementation)**
4. **Ball of Mud 오류 enum 피하기 (Avoiding Ball of Mud Error Enums)**
5. **오류 기록/로깅 (Error Logging & Context)**

---

## 1. 제어 흐름 오류 처리 (Control Flow Error Handling)

### 목적
프로그램의 정상적인 흐름을 관리하기 위해 `Result<T, E>` 타입을 사용합니다.

### 구현

#### 도메인별 오류 타입 (Domain-Specific Error Types)

```rust
// src/error.rs에서 정의

// 검증 오류
pub enum ValidationError {
    EmptyField(String),
    TooShort(String, usize),
    TooLong(String, usize),
    InvalidFormat(String),
    SuspiciousContent(String),
    PossibleSQLInjection,
}

// 데이터베이스 오류
pub enum DatabaseError {
    UniqueConstraintViolation(String),
    NotFound(String),
    QueryExecution(String),
    ConnectionPool(String),
    UnexpectedError(String),
}

// 이메일 서비스 오류
pub enum EmailError {
    SendFailed(String),
    InvalidRecipient(String),
    ServiceUnavailable(String),
    ConfigurationError(String),
}
```

#### 통합 오류 타입 (Unified Error Type)

```rust
pub enum AppError {
    Validation(ValidationError),
    Database(DatabaseError),
    Email(EmailError),
    Config(ConfigError),
    Internal(String),
}
```

### 사용 예시

#### validators.rs 모듈

```rust
pub fn is_valid_email(email: &str) -> Result<String, ValidationError> {
    let trimmed = email.trim();

    if trimmed.is_empty() {
        return Err(ValidationError::EmptyField("email".to_string()));
    }

    if trimmed.len() > MAX_EMAIL_LENGTH {
        return Err(ValidationError::TooLong("email".to_string(), MAX_EMAIL_LENGTH));
    }

    if !EMAIL_REGEX.is_match(trimmed) {
        return Err(ValidationError::InvalidFormat("email".to_string()));
    }

    Ok(trimmed.to_string())
}
```

#### routes/subscriptions.rs에서의 사용

```rust
pub async fn subscribe(
    form: web::Form<FormData>,
    pool: web::Data<PgPool>,
    email_client: web::Data<EmailClient>,
) -> Result<HttpResponse, AppError> {
    // ? 연산자를 사용한 자동 오류 전파
    let name = form.name.as_ref()
        .ok_or_else(|| AppError::Validation(
            ValidationError::EmptyField("name".to_string())
        ))?;
    let name = is_valid_name(name)?;  // ValidationError가 자동으로 AppError로 변환

    let email = form.email.as_ref()
        .ok_or_else(|| AppError::Validation(
            ValidationError::EmptyField("email".to_string())
        ))?;
    let email = is_valid_email(email)?;  // 자동 변환

    // 데이터베이스 작업 - sqlx::Error가 자동으로 AppError로 변환
    create_subscriber(&pool, subscriber_id, &email, &name, &error_context).await?;

    Ok(HttpResponse::Ok().finish())
}
```

### 장점

- **타입 안전성**: 컴파일 시간에 모든 오류 경로 검사
- **명시적 오류 처리**: 오류 무시 불가능
- **자동 변환**: `From` 트레이트로 오류 타입 자동 변환
- **간결한 문법**: `?` 연산자로 깔끔한 코드

---

## 2. 운영자를 위한 오류 처리 (Operator/System Error Handling)

### 목적
사용자와 운영자에게 의미 있는 HTTP 응답을 제공합니다.

### HTTP 응답 매핑

```rust
impl ErrorHandler for AppError {
    fn error_response(&self, request_id: &str) -> (StatusCode, ErrorResponse) {
        match self {
            // 400 Bad Request - 검증 오류
            AppError::Validation(e) => (
                StatusCode::BAD_REQUEST,
                "VALIDATION_ERROR".to_string(),
                e.to_string(),
            ),

            // 409 Conflict - 중복 항목
            AppError::Database(DatabaseError::UniqueConstraintViolation(_)) => (
                StatusCode::CONFLICT,
                "DUPLICATE_ENTRY".to_string(),
                e.to_string(),
            ),

            // 404 Not Found - 찾을 수 없음
            AppError::Database(DatabaseError::NotFound(_)) => (
                StatusCode::NOT_FOUND,
                "NOT_FOUND".to_string(),
                e.to_string(),
            ),

            // 503 Service Unavailable - 서비스 이용 불가
            AppError::Email(_) => (
                StatusCode::SERVICE_UNAVAILABLE,
                "EMAIL_SERVICE_ERROR".to_string(),
                "Email service temporarily unavailable".to_string(),
            ),

            // 500 Internal Server Error - 내부 오류
            AppError::Internal(_) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "INTERNAL_ERROR".to_string(),
                "Internal server error".to_string(),
            ),
        }
    }
}
```

### 오류 응답 구조

```rust
#[derive(Serialize)]
pub struct ErrorResponse {
    pub error_id: String,           // 고유 오류 추적 ID
    pub message: String,             // 사용자 친화적 메시지
    pub code: String,                // 클라이언트 처리용 오류 코드
    pub status: u16,                 // HTTP 상태 코드
    pub timestamp: String,           // ISO 8601 형식 타임스탬프
}
```

### JSON 응답 예시

```json
// 검증 오류
{
    "error_id": "123e4567-e89b-12d3-a456-426614174000",
    "message": "email is too long (maximum 254 characters)",
    "code": "VALIDATION_ERROR",
    "status": 400,
    "timestamp": "2024-11-22T10:30:45.123Z"
}

// 중복 항목
{
    "error_id": "123e4567-e89b-12d3-a456-426614174001",
    "message": "Duplicate entry: Email already registered",
    "code": "DUPLICATE_ENTRY",
    "status": 409,
    "timestamp": "2024-11-22T10:30:46.123Z"
}

// 서비스 이용 불가
{
    "error_id": "123e4567-e89b-12d3-a456-426614174002",
    "message": "Email service temporarily unavailable",
    "code": "EMAIL_SERVICE_ERROR",
    "status": 503,
    "timestamp": "2024-11-22T10:30:47.123Z"
}
```

### Actix-web 통합

```rust
// ResponseError 트레이트 구현
impl ResponseError for AppError {
    fn error_response(&self) -> HttpResponse {
        let request_id = uuid::Uuid::new_v4().to_string();
        self.log_error(&request_id);

        let (status, error_response) =
            <Self as ErrorHandler>::error_response(self, &request_id);

        HttpResponse::build(status).json(error_response)
    }

    fn status_code(&self) -> StatusCode {
        // 상태 코드 결정 로직
    }
}

// 라우트에서 자동 사용
pub async fn subscribe(...) -> Result<HttpResponse, AppError> {
    // AppError 반환 시 자동으로 JSON 응답으로 변환
}
```

---

## 3. Error 트레이트 구현 (Error Trait Implementation)

### 표준 Error 트레이트

```rust
use std::error::Error as StdError;
use std::fmt;

pub enum ValidationError { /* ... */ }

// Display 트레이트 구현 (필수)
impl fmt::Display for ValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ValidationError::EmptyField(field) =>
                write!(f, "{} is empty", field),
            ValidationError::TooShort(field, min) =>
                write!(f, "{} is too short (minimum {} characters)", field, min),
            // ...
        }
    }
}

// Error 트레이트 구현 (자동)
impl StdError for ValidationError {}
```

### Debug 트레이트

모든 오류 타입에 `#[derive(Debug)]` 자동 적용:

```rust
#[derive(Debug)]
pub enum ValidationError {
    EmptyField(String),
    // ...
}
```

### From 트레이트를 통한 오류 변환

```rust
// ValidationError -> AppError 변환
impl From<ValidationError> for AppError {
    fn from(err: ValidationError) -> Self {
        AppError::Validation(err)
    }
}

// sqlx::Error -> AppError 변환
impl From<sqlx::Error> for AppError {
    fn from(err: sqlx::Error) -> Self {
        let error_msg = err.to_string();

        if error_msg.contains("duplicate key") {
            AppError::Database(DatabaseError::UniqueConstraintViolation(
                "Email already registered".to_string(),
            ))
        } else if error_msg.contains("no rows") {
            AppError::Database(DatabaseError::NotFound(
                "Record not found".to_string(),
            ))
        } else {
            AppError::Database(DatabaseError::UnexpectedError(error_msg))
        }
    }
}
```

---

## 4. Ball of Mud 오류 Enum 피하기

### 문제점

하나의 거대한 오류 enum은 관리가 어렵습니다:

```rust
// ❌ 피해야 할 패턴
pub enum Error {
    ValidationError(String),
    DatabaseError(String),
    EmailError(String),
    ConfigError(String),
    // 수백 개의 변형...
    Unknown,
}
```

### 해결책: 계층화된 오류 타입

```rust
// ✅ 권장 패턴
// 1단계: 도메인별 오류
pub enum ValidationError { /* ... */ }
pub enum DatabaseError { /* ... */ }
pub enum EmailError { /* ... */ }

// 2단계: 통합 오류 (필요한 경우에만)
pub enum AppError {
    Validation(ValidationError),
    Database(DatabaseError),
    Email(EmailError),
    // ...
}
```

### 장점

- **명확한 책임**: 각 모듈이 자신의 오류만 정의
- **재사용성**: 특정 오류 타입을 필요한 곳에서만 사용
- **유지보수성**: 새 오류 추가 시 한 곳만 수정
- **타입 안전성**: 불필요한 변환 제거

### 실제 사용 예시

```rust
// validation 모듈에서
pub fn validate(input: &str) -> Result<String, ValidationError> {
    // ValidationError만 처리
}

// email 모듈에서
pub async fn send(to: &str) -> Result<(), EmailError> {
    // EmailError만 처리
}

// routes 모듈에서
pub async fn handler(...) -> Result<HttpResponse, AppError> {
    // 필요시 AppError로 변환
    let validated = validate(input)?;  // ValidationError -> AppError
    send(&email).await?;               // EmailError -> AppError
}
```

---

## 5. 오류 기록/로깅 (Error Logging & Context)

### ErrorContext 구조

```rust
pub struct ErrorContext {
    pub request_id: String,          // 요청 고유 ID
    pub user_id: Option<String>,     // 사용자 ID
    pub operation: String,            // 작업 이름
    pub timestamp: DateTime<Utc>,     // 타임스탬프
}

impl ErrorContext {
    pub fn new(operation: impl Into<String>) -> Self {
        Self {
            request_id: Uuid::new_v4().to_string(),
            user_id: None,
            operation: operation.into(),
            timestamp: Utc::now(),
        }
    }

    pub fn with_user_id(mut self, user_id: String) -> Self {
        self.user_id = Some(user_id);
        self
    }
}
```

### 로깅 레벨별 전략

```rust
impl ErrorHandler for AppError {
    fn log_error(&self, request_id: &str) {
        match self {
            // WARN: 사용자 입력 오류 (예상되는 오류)
            AppError::Validation(e) => {
                tracing::warn!(
                    request_id = request_id,
                    error = %e,
                    "Validation error"
                );
            }

            // WARN: 중복 항목 (일반적인 비즈니스 로직)
            AppError::Database(DatabaseError::UniqueConstraintViolation(_)) => {
                tracing::warn!(
                    request_id = request_id,
                    error = %self,
                    "Duplicate entry attempt"
                );
            }

            // ERROR: 시스템 오류 (예상치 못한 오류)
            AppError::Database(e) => {
                tracing::error!(
                    request_id = request_id,
                    error = %e,
                    "Database error"
                );
            }

            AppError::Email(e) => {
                tracing::error!(
                    request_id = request_id,
                    error = %e,
                    "Email service error"
                );
            }
        }
    }
}
```

### 구조화된 로깅 예시

```rust
// routes/subscriptions.rs
async fn create_subscriber(
    pool: &web::Data<PgPool>,
    subscriber_id: Uuid,
    email: &str,
    name: &str,
    context: &ErrorContext,
) -> Result<(), AppError> {
    sqlx::query(
        "INSERT INTO subscriptions (id, email, name, subscribed_at, status) VALUES ($1, $2, $3, $4, $5)"
    )
    .bind(subscriber_id)
    .bind(email)
    .bind(name)
    .bind(Utc::now())
    .bind("pending")
    .execute(pool.get_ref())
    .await
    .map_err(|e| {
        let error = AppError::from(e);
        context.log_error(&error);  // 구조화된 로깅
        error
    })?;

    tracing::info!(
        request_id = %context.request_id,
        subscriber_id = %subscriber_id,
        "New subscriber saved successfully"
    );

    Ok(())
}
```

### JSON 구조화 로깅 출력

```json
{
    "timestamp": "2024-11-22T10:30:45.123Z",
    "level": "WARN",
    "message": "Validation error",
    "request_id": "123e4567-e89b-12d3-a456-426614174000",
    "error": "email is too long (maximum 254 characters)",
    "target": "zero2prod::routes::subscriptions"
}
```

```json
{
    "timestamp": "2024-11-22T10:30:46.123Z",
    "level": "ERROR",
    "message": "Database error",
    "request_id": "123e4567-e89b-12d3-a456-426614174001",
    "error": "connection timeout",
    "target": "zero2prod::error"
}
```

---

## 사용 패턴 비교

### 라우트 핸들러 전후 비교

#### Before (이전)
```rust
pub async fn subscribe(
    form: web::Form<FormData>,
    pool: web::Data<PgPool>,
    email_client: web::Data<EmailClient>,
) -> HttpResponse {
    // 수동 오류 처리
    let name = match form.name.as_ref() {
        Some(n) => match is_valid_name(n) {
            Ok(validated) => validated,
            Err(e) => {
                tracing::warn!(error = %e, "Invalid name");
                return HttpResponse::BadRequest().finish();
            }
        },
        None => {
            tracing::warn!("Missing name");
            return HttpResponse::BadRequest().finish();
        }
    };

    match sqlx::query(...).execute(...).await {
        Ok(_) => { /* success */ },
        Err(e) => {
            let error_message = e.to_string();
            if error_message.contains("duplicate key") {
                return HttpResponse::Conflict().finish();
            }
            return HttpResponse::InternalServerError().finish();
        }
    }
}
```

#### After (이후)
```rust
pub async fn subscribe(
    form: web::Form<FormData>,
    pool: web::Data<PgPool>,
    email_client: web::Data<EmailClient>,
) -> Result<HttpResponse, AppError> {
    let error_context = ErrorContext::new("subscription_creation");

    let name = form.name.as_ref()
        .ok_or_else(|| AppError::Validation(
            ValidationError::EmptyField("name".to_string())
        ))?;
    let name = is_valid_name(name)?;

    let email = form.email.as_ref()
        .ok_or_else(|| AppError::Validation(
            ValidationError::EmptyField("email".to_string())
        ))?;
    let email = is_valid_email(email)?;

    create_subscriber(&pool, subscriber_id, &email, &name, &error_context).await?;

    Ok(HttpResponse::Ok().finish())
}
```

---

## 체크리스트

### 새로운 오류 타입 추가 시

- [ ] 도메인별 오류 enum 정의 (`error.rs`)
- [ ] `Display` 트레이트 구현
- [ ] `StdError` 트레이트 구현
- [ ] `From` 트레이트로 변환 구현
- [ ] 적절한 HTTP 상태 코드 매핑
- [ ] 로깅 레벨 설정 (WARN/ERROR)
- [ ] 단위 테스트 작성

### 라우트 구현 시

- [ ] `Result<HttpResponse, AppError>` 반환 타입 사용
- [ ] `ErrorContext` 생성
- [ ] `?` 연산자로 오류 전파
- [ ] 필요시 `context.log_error()` 호출
- [ ] 성공 경로 로깅

---

## 참고 자료

- `src/error.rs` - 모든 오류 타입과 구현
- `src/validators.rs` - 검증 오류 사용 예시
- `src/routes/subscriptions.rs` - 통합 오류 사용 예시
- `src/routes/confirmation.rs` - 데이터베이스 오류 처리 예시
- `src/email_client.rs` - 이메일 서비스 오류 처리 예시
