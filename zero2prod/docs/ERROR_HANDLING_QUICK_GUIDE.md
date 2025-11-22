# 오류 처리 빠른 참고서 (Error Handling Quick Reference)

## 파일 위치

- **오류 타입 정의**: `src/error.rs`
- **검증 오류**: `src/validators.rs`
- **이메일 오류**: `src/email_client.rs`
- **라우트 사용 예시**: `src/routes/subscriptions.rs`, `src/routes/confirmation.rs`

---

## 1. 새로운 검증 함수 추가

```rust
// src/validators.rs 또는 새 모듈에서
use crate::error::ValidationError;

pub fn validate_something(input: &str) -> Result<String, ValidationError> {
    if input.is_empty() {
        return Err(ValidationError::EmptyField("field_name".to_string()));
    }

    if input.len() > 100 {
        return Err(ValidationError::TooLong("field_name".to_string(), 100));
    }

    Ok(input.to_string())
}
```

## 2. 라우트에서 오류 처리

```rust
// src/routes/new_endpoint.rs
use crate::error::{AppError, ErrorContext};

pub async fn handler(
    form: web::Form<FormData>,
    pool: web::Data<PgPool>,
) -> Result<HttpResponse, AppError> {
    let error_context = ErrorContext::new("operation_name");

    // 검증 (자동으로 AppError로 변환됨)
    let validated = validate_something(&input)?;

    // 데이터베이스 작업 (sqlx::Error가 자동으로 AppError로 변환됨)
    sqlx::query(...)
        .execute(pool.get_ref())
        .await
        .map_err(|e| {
            let error = AppError::from(e);
            error_context.log_error(&error);
            error
        })?;

    Ok(HttpResponse::Ok().finish())
}
```

## 3. 새로운 오류 타입 추가

### 단계 1: Enum 정의

```rust
// src/error.rs에 추가
#[derive(Debug)]
pub enum NewDomainError {
    SpecificError(String),
    AnotherError(String),
}

impl fmt::Display for NewDomainError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            NewDomainError::SpecificError(msg) =>
                write!(f, "Specific error: {}", msg),
            NewDomainError::AnotherError(msg) =>
                write!(f, "Another error: {}", msg),
        }
    }
}

impl StdError for NewDomainError {}
```

### 단계 2: AppError에 추가

```rust
// src/error.rs의 AppError enum에 추가
pub enum AppError {
    Validation(ValidationError),
    Database(DatabaseError),
    Email(EmailError),
    Config(ConfigError),
    NewDomain(NewDomainError),  // 새로 추가
    Internal(String),
}
```

### 단계 3: From 트레이트 구현

```rust
impl From<NewDomainError> for AppError {
    fn from(err: NewDomainError) -> Self {
        AppError::NewDomain(err)
    }
}
```

### 단계 4: HTTP 매핑 추가

```rust
// src/error.rs의 ErrorHandler impl에 추가
AppError::NewDomain(e) => (
    StatusCode::BAD_REQUEST,  // 적절한 상태 코드 선택
    "NEW_DOMAIN_ERROR".to_string(),
    e.to_string(),
),
```

### 단계 5: 로깅 추가

```rust
// src/error.rs의 log_error 메서드에 추가
AppError::NewDomain(e) => {
    tracing::warn!(
        request_id = request_id,
        error = %e,
        "New domain error"
    );
}
```

---

## HTTP 상태 코드 매핑 가이드

| 오류 타입 | HTTP 상태 | 설명 |
|----------|----------|------|
| ValidationError | 400 | 클라이언트 입력 오류 |
| UniqueConstraintViolation | 409 | 중복된 리소스 |
| NotFound | 404 | 리소스를 찾을 수 없음 |
| EmailError | 503 | 외부 서비스 이용 불가 |
| ConnectionPool | 503 | 데이터베이스 연결 불가 |
| Internal/Config | 500 | 서버 오류 |

---

## 로깅 레벨 가이드

```rust
// WARN - 예상되는 오류 또는 일반적인 비즈니스 로직 오류
tracing::warn!(
    request_id = request_id,
    error = %e,
    "User input validation failed"
);

// ERROR - 예상치 못한 시스템 오류
tracing::error!(
    request_id = request_id,
    error = %e,
    "Database connection failed"
);

// INFO - 성공 경로 또는 중요한 이벤트
tracing::info!(
    request_id = request_id,
    subscriber_id = %id,
    "Subscription created successfully"
);
```

---

## 테스트 작성 예시

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validation_error() {
        let result = validate_something("");
        assert!(result.is_err());

        match result {
            Err(ValidationError::EmptyField(_)) => (),
            _ => panic!("Expected EmptyField error"),
        }
    }

    #[test]
    fn test_app_error_conversion() {
        let val_err = ValidationError::InvalidFormat("test".to_string());
        let app_err: AppError = val_err.into();

        match app_err {
            AppError::Validation(_) => (),
            _ => panic!("Expected Validation error"),
        }
    }
}
```

---

## 자주 사용되는 패턴

### 패턴 1: 선택 사항 필드 검증

```rust
let name = form.name.as_ref()
    .ok_or_else(|| AppError::Validation(
        ValidationError::EmptyField("name".to_string())
    ))?;
let name = is_valid_name(name)?;
```

### 패턴 2: 다중 오류 로깅과 함께 변환

```rust
sqlx::query(...)
    .execute(pool.get_ref())
    .await
    .map_err(|e| {
        let error = AppError::from(e);
        error_context.log_error(&error);
        error
    })?;
```

### 패턴 3: 커스텀 오류 메시지

```rust
result.map_err(|e| {
    AppError::Database(DatabaseError::UnexpectedError(
        format!("Failed to save token: {}", e)
    ))
})?;
```

### 패턴 4: 조건부 오류 생성

```rust
if rows_affected == 0 {
    return Err(AppError::Database(
        DatabaseError::NotFound("Record not found".to_string())
    ));
}
```

---

## 디버깅 팁

### 오류 체인 추적

```rust
// 각 함수에서 error_context를 전달하면
// 모든 오류가 요청 ID와 함께 로깅됨
pub async fn create_subscriber(
    pool: &web::Data<PgPool>,
    email: &str,
    context: &ErrorContext,
) -> Result<(), AppError> {
    sqlx::query(...)
        .await
        .map_err(|e| {
            let error = AppError::from(e);
            context.log_error(&error);  // 요청 ID 포함 로깅
            error
        })?;
}
```

### 구조화된 로그에서 검색

```bash
# 특정 request_id로 모든 오류 찾기
grep "request_id=123e4567-e89b-12d3-a456-426614174000" application.log

# 특정 작업의 모든 오류 찾기
grep "operation=subscription_creation" application.log | grep ERROR

# 이메일 오류만 필터링
grep "EMAIL_SERVICE_ERROR" application.log
```

---

## 체크리스트

새 기능을 추가할 때:

- [ ] ValidationError enum에 변형 추가 (필요시)
- [ ] 검증 함수 작성 및 테스트
- [ ] 라우트 핸들러에서 `Result<HttpResponse, AppError>` 반환
- [ ] ErrorContext 생성
- [ ] `?` 연산자로 오류 전파
- [ ] 성공/실패 경로 로깅
- [ ] 모든 테스트 통과 확인
- [ ] 설명서 업데이트 (필요시)

---

## 참고 자료

더 자세한 내용은 `docs/ERROR_HANDLING.md`를 참조하세요.
