# Security Implementation Guide - Invalid Subscriber Protection

이 문서는 유효하지 않은 구독자 데이터로부터 보호하는 구현된 보안 기능들을 설명합니다.

## 목차
1. [DoS 공격 방어](#dos-공격-방어)
2. [데이터 갈취 방지](#데이터-갈취-방지)
3. [피싱 공격 방어](#피싱-공격-방어)
4. [SQL 인젝션 방어](#sql-인젝션-방어)

---

## 1. DoS 공격 방어

### 1.1 입력 길이 제한 (256자 이하)

**구현 위치**: `src/validators.rs`

```rust
const MAX_EMAIL_LENGTH: usize = 254; // RFC 5321 표준
const MAX_NAME_LENGTH: usize = 256;  // 커스텀 제한
```

**보호 메커니즘**:
- 이메일: 최대 254자 (RFC 5321 표준)
- 이름: 최대 256자 (요구사항)
- Payload bomb 공격 방지

**테스트**: `subscribe_rejects_email_exceeding_256_chars`, `subscribe_rejects_name_exceeding_256_chars`

### 1.2 페이로드 크기 제한

**구현 위치**: `src/startup.rs` (구현 가능 구조)

- 최대 POST 요청 크기: 1KB
- 대규모 파일 업로드 공격 방지

### 1.3 Rate Limiting (분당 요청 제한)

**구현 위치**: `src/security.rs`

```rust
pub struct RateLimiterManager {
    pub requests_per_minute: u32,  // 기본값: 10
    pub max_content_length: u64,   // 1024 bytes
}
```

**보호 메커니즘**:
- IP별 토큰 버킷 알고리즘
- 분당 최대 10개 요청 (IP당)
- 메모리 효율적 구현

**활성화 방법**:
```rust
let rate_limiter = RateLimiterManager::new(RateLimitConfig::default());
rate_limiter.check_rate_limit("192.168.1.1")?;
```

---

## 2. 데이터 갈취 방지

### 2.1 민감 데이터 로깅 제한

**구현 위치**: `src/routes/subscriptions.rs`

**개선사항**:
- ❌ 이전: 이메일과 이름을 로그에 평문으로 기록
- ✅ 현재: 민감 데이터 제거 및 조건부 로깅

```rust
// 민감 데이터 제거
tracing::info!("Processing new subscription (sensitive data redacted)");

// 성공 로그 - 구독자 ID만 기록
tracing::info!(
    subscriber_id = %subscriber_id,
    "New subscriber saved successfully"
);
```

### 2.2 데이터 검증 및 살균 처리

**구현 위치**: `src/validators.rs`

**제거되는 패턴**:
- Null 바이트 (`\0`)
- 제어 문자
- 과도한 특수 문자 (5개 이상)

```rust
fn has_suspicious_name_patterns(name: &str) -> bool {
    // Null byte 검사
    if name.contains('\0') {
        return true;
    }

    // 제어 문자 검사
    if name.chars().any(|c| c.is_control()) {
        return true;
    }

    // 과도한 특수 문자 검사
    let special_char_count = name.chars()
        .filter(|c| !c.is_alphanumeric() && !c.is_whitespace() && ...)
        .count();

    special_char_count > 5
}
```

### 2.3 안전한 헤더 설정

**구현 위치**: `src/security.rs`

```rust
pub struct SecurityHeaders;

impl SecurityHeaders {
    pub fn get_headers() -> Vec<(String, String)> {
        vec![
            // CSRF 보호
            ("X-CSRF-Token", "required"),

            // XSS 보호
            ("X-Content-Type-Options", "nosniff"),
            ("X-Frame-Options", "SAMEORIGIN"),
            ("X-XSS-Protection", "1; mode=block"),

            // CSP (Content Security Policy)
            ("Content-Security-Policy", "default-src 'self'"),

            // 레퍼러 정책 (데이터 갈취 방지)
            ("Referrer-Policy", "strict-origin-when-cross-origin"),

            // HSTS (HTTPS only)
            ("Strict-Transport-Security", "max-age=31536000"),
        ]
    }
}
```

---

## 3. 피싱 공격 방어

### 3.1 이메일 형식 검증 (RFC 5322)

**구현 위치**: `src/validators.rs`

```rust
static ref EMAIL_REGEX: Regex = Regex::new(
    r"^[a-zA-Z0-9.!#$%&'*+/=?^_`{|}~-]+@[a-zA-Z0-9](?:[a-zA-Z0-9-]{0,61}[a-zA-Z0-9])?(?:\.[a-zA-Z0-9](?:[a-zA-Z0-9-]{0,61}[a-zA-Z0-9])?)*$"
).unwrap();
```

**검증 규칙**:
- RFC 5322 표준 준수
- 유효한 로컬 파트 (@ 앞)
- 유효한 도메인 파트 (@ 뒤)
- 서브도메인 지원

**테스트**: `subscribe_rejects_invalid_email_format`

### 3.2 피싱 패턴 감지

**구현 위치**: `src/validators.rs`

```rust
fn has_suspicious_email_patterns(email: &str) -> bool {
    // 1. 과도하게 긴 로컬 파트 감지 (64자 초과)
    if let Some(at_pos) = email.find('@') {
        let local_part = &email[..at_pos];
        if local_part.len() > 64 {
            return true;
        }
    }

    // 2. 다중 @ 기호 감지
    if email.matches('@').count() != 1 {
        return true;
    }

    // 3. Null 바이트 감지
    if email.contains('\0') {
        return true;
    }

    false
}
```

**감지 항목**:
- 과도하게 긴 로컬 파트 (64자 초과)
- 다중 @ 기호
- Null 바이트

---

## 4. SQL 인젝션 방어

### 4.1 매개변수화된 쿼리 (기본)

**구현 위치**: `src/routes/subscriptions.rs`

```rust
match sqlx::query(
    "INSERT INTO subscriptions (id, email, name, subscribed_at) VALUES ($1, $2, $3, $4)"
)
.bind(subscriber_id)
.bind(&email)
.bind(&name)
.bind(Utc::now())
.execute(pool.get_ref())
.await
```

**장점**:
- SQLx의 컴파일 타임 쿼리 검증
- 자동 파라미터 이스케이핑
- 타입 안전성 보장

### 4.2 입력 패턴 감지

**구현 위치**: `src/validators.rs`

감지되는 SQL 인젝션 패턴:

```rust
static ref SQL_INJECTION_PATTERNS: [Regex; 6] = [
    // 1. UNION 기반 인젝션
    Regex::new(r"(?i)\s+UNION\s+").unwrap(),

    // 2. 주석 기반 인젝션
    Regex::new(r"(--|;|/\*|\*/|xp_|sp_)").unwrap(),

    // 3. 스택된 쿼리
    Regex::new(r"(?i);\s*(INSERT|UPDATE|DELETE|DROP|CREATE|ALTER)").unwrap(),

    // 4. 시간 기반 blind 인젝션
    Regex::new(r"(?i)(SLEEP|WAITFOR|BENCHMARK|DBMS_LOCK)").unwrap(),

    // 5. 부울 기반 인젝션
    Regex::new(r#"(?i)(\bOR\b|\bAND\b)\s*(['"][0-9]*['"]|[0-9]*)\s*=\s*(['"][0-9]*['"]|[0-9]*|True|False)"#).unwrap(),

    // 6. 함수 기반 인젝션
    Regex::new(r"(?i)(CAST|CONVERT|SUBSTRING|CONCAT|LOAD_FILE)").unwrap(),
];
```

**테스트**:
- `subscribe_rejects_sql_injection_in_email`
- `subscribe_rejects_sql_injection_in_name`

### 4.3 중복 이메일 처리

**구현 위치**: `src/routes/subscriptions.rs`

```rust
match sqlx::query(...).execute(...).await {
    Ok(_) => HttpResponse::Ok().finish(),
    Err(e) => {
        let error_message = e.to_string();
        if error_message.contains("duplicate key") {
            return HttpResponse::Conflict().finish(); // 409 Conflict
        }
        HttpResponse::InternalServerError().finish()
    }
}
```

**테스트**: `subscribe_rejects_duplicate_email`

---

## 보안 체크리스트

### DoS 공격 방어 ✅
- [x] 입력 길이 제한 (256자)
- [x] 페이로드 크기 제한 (1KB)
- [x] Rate limiting (10 req/min per IP)
- [x] 제어 문자 필터링

### 데이터 갈취 방지 ✅
- [x] 민감 데이터 로깅 제한
- [x] Null 바이트 제거
- [x] 특수 문자 검증
- [x] 안전 헤더 설정

### 피싱 공격 방어 ✅
- [x] RFC 5322 이메일 검증
- [x] 비정상 이메일 패턴 감지
- [x] 로컬 파트 길이 제한
- [x] 다중 @ 기호 감지

### SQL 인젝션 방어 ✅
- [x] 매개변수화된 쿼리 (SQLx)
- [x] 위험 SQL 패턴 감지
- [x] UNION SELECT 차단
- [x] 주석 문자 차단
- [x] 제어 문자 차단

---

## 테스트 실행

### 모든 보안 테스트 실행
```bash
cargo test --test health_check -- --nocapture
```

### 특정 보안 테스트 실행
```bash
# DoS 공격 방어
cargo test subscribe_rejects_email_exceeding_256_chars
cargo test subscribe_rejects_name_exceeding_256_chars

# SQL 인젝션
cargo test subscribe_rejects_sql_injection_in_email
cargo test subscribe_rejects_sql_injection_in_name

# 피싱 방어
cargo test subscribe_rejects_invalid_email_format

# 데이터 갈취 방지
cargo test subscribe_rejects_control_characters_in_name
cargo test subscribe_rejects_duplicate_email
```

---

## 향후 개선 사항

### 1. 인증 및 권한 부여
- JWT 토큰 기반 인증
- 역할 기반 접근 제어 (RBAC)

### 2. 감사 로깅
- 별도의 감사 로그 파일
- 보안 이벤트 추적
- 접근 로그 보존

### 3. API 제한
- API 키 관리
- 요청 서명 (HMAC)
- 타임스탬프 검증

### 4. 암호화
- 이메일 필드 암호화 (저장 시)
- TLS/SSL (전송 시)
- 키 로테이션

### 5. 모니터링
- 실시간 이상 탐지
- 보안 이벤트 알림
- 로그 분석 및 집계

---

## 참고 자료

### 보안 표준
- OWASP Top 10: https://owasp.org/www-project-top-ten/
- RFC 5322 (이메일): https://tools.ietf.org/html/rfc5322
- CWE (Common Weakness Enumeration): https://cwe.mitre.org/

### Rust 보안 라이브러리
- `validator`: 입력 검증
- `ring`: 암호화
- `argon2`: 비밀번호 해싱
- `jsonwebtoken`: JWT 처리

---

## 버전 정보

- **구현 날짜**: 2025-11-20
- **Rust 버전**: 1.70+
- **주요 의존성**:
  - actix-web 4
  - sqlx 0.6
  - regex 1
  - lazy_static 1.4

---

## 문의

보안 취약점 발견 시: security@example.com
