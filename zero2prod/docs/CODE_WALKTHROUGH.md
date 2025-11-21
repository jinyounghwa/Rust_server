# 이메일 확인 서비스 - 코드 상세 설명

## 📖 목차

1. [이메일 클라이언트 상세 설명](#1-이메일-클라이언트-상세-설명)
2. [확인 토큰 상세 설명](#2-확인-토큰-상세-설명)
3. [확인 엔드포인트 상세 설명](#3-확인-엔드포인트-상세-설명)
4. [구독 엔드포인트 통합](#4-구독-엔드포인트-통합)
5. [데이터베이스 스키마](#5-데이터베이스-스키마)

---

## 1. 이메일 클라이언트 상세 설명

### 파일 위치
```
src/email_client.rs
```

### 1.1 전체 코드 구조

```rust
// 1. 라이브러리 임포트
use crate::validators::is_valid_email;
use serde::Serialize;

// 2. EmailClient 구조체 정의
#[derive(Clone)]
pub struct EmailClient { ... }

// 3. ConfirmedSubscriber 구조체 정의
#[derive(Clone)]
pub struct ConfirmedSubscriber(String);

// 4. SendEmailRequest 구조체 정의
#[derive(Serialize)]
pub struct SendEmailRequest { ... }

// 5. 구현 블록
impl EmailClient { ... }
impl ConfirmedSubscriber { ... }

// 6. 단위 테스트
#[cfg(test)]
mod tests { ... }
```

### 1.2 EmailClient 구조체 상세 분석

#### 구조체 정의

```rust
#[derive(Clone)]
pub struct EmailClient {
    http_client: reqwest::Client,
    base_url: String,
    sender: ConfirmedSubscriber,
}
```

**필드 설명:**

| 필드 | 타입 | 목적 | 예시 |
|------|------|------|------|
| `http_client` | `reqwest::Client` | HTTP 요청 수행 | 비동기 HTTP POST |
| `base_url` | `String` | 이메일 서버 URL | `http://localhost:3030` |
| `sender` | `ConfirmedSubscriber` | 검증된 발신자 이메일 | `noreply@example.com` |

**#[derive(Clone)] 매크로:**
- 자동으로 `Clone` 트레이트 구현
- 다른 스레드/핸들러에서 복제 가능
- Actix-web의 `web::Data<T>`와 호환

#### new() 메서드

```rust
pub fn new(
    base_url: String,
    sender: ConfirmedSubscriber,
    http_client: reqwest::Client,
) -> Self {
    Self {
        http_client,
        base_url,
        sender,
    }
}
```

**사용 방법:**

```rust
let email_client = EmailClient::new(
    "http://localhost:3030".to_string(),
    ConfirmedSubscriber::parse("noreply@example.com".to_string())?,
    reqwest::Client::new(),
);
```

### 1.3 ConfirmedSubscriber 상세 분석

#### Newtype 패턴

```rust
#[derive(Clone)]
pub struct ConfirmedSubscriber(String);
```

**왜 이렇게 설계했는가?**

```rust
// ❌ 문제: String 그대로 사용
pub fn send_email(&self, recipient: String) {
    // recipient가 유효한 이메일인지 알 수 없음
}

// ✅ 해결: ConfirmedSubscriber 타입 사용
pub fn send_email(&self, recipient: ConfirmedSubscriber) {
    // 타입 시스템이 검증됨을 보장
}
```

**Newtype 패턴의 장점:**
- 타입 안전성
- 컴파일 타임 검증
- 런타임 오버헤드 없음 (제로 코스트 추상화)

#### parse() 메서드

```rust
pub fn parse(s: String) -> Result<Self, String> {
    let email = is_valid_email(&s)
        .map_err(|e| format!("{:?}", e))?;
    Ok(Self(email))
}
```

**단계별 분석:**

```
입력: "test@example.com"
  │
  ├─ is_valid_email(&s) 호출
  │  │
  │  └─ Result<String, ValidationError> 반환
  │     │
  │     ├─ Ok("test@example.com") 또는
  │     └─ Err(ValidationError)
  │
  ├─ .map_err(|e| format!("{:?}", e))
  │  │
  │  └─ Err를 String으로 변환
  │
  ├─ ? 연산자
  │  │
  │  ├─ Ok면: 값 추출, 계속 진행
  │  └─ Err면: 함수 반환
  │
  └─ Ok(Self(email)) 반환
```

**사용 예:**

```rust
// 성공 케이스
match ConfirmedSubscriber::parse("john@example.com".to_string()) {
    Ok(subscriber) => {
        // subscriber 사용 가능
        println!("{}", subscriber.inner());  // john@example.com
    }
    Err(e) => println!("Invalid: {}", e),
}

// 실패 케이스
match ConfirmedSubscriber::parse("invalid-email".to_string()) {
    Ok(subscriber) => { /* 도달 불가 */ }
    Err(e) => println!("Invalid: {}", e),  // "Invalid email format"
}
```

#### inner() 메서드

```rust
pub fn inner(&self) -> &str {
    &self.0
}
```

**왜 메서드로 제공하는가?**

```rust
// ❌ 나쁜 설계: 직접 접근
pub email: String

// ✅ 좋은 설계: 메서드 제공
pub fn inner(&self) -> &str { &self.0 }

// 이유: 나중에 검증 로직 추가 가능
pub fn inner(&self) -> &str {
    // 캐싱, 로깅 등 추가 가능
    &self.0
}
```

### 1.4 SendEmailRequest 상세 분석

#### 구조체 정의

```rust
#[derive(Serialize)]
pub struct SendEmailRequest {
    to: String,
    #[serde(rename = "Html")]
    html: String,
    #[serde(rename = "Subject")]
    subject: String,
}
```

**필드 매핑:**

| Rust 필드 | JSON 필드 | 타입 | 예시 |
|----------|----------|------|------|
| `to` | `to` | String | `john@example.com` |
| `html` | `Html` | String | `<h1>Welcome!</h1>` |
| `subject` | `Subject` | String | `Confirm your email` |

**#[serde(rename)] 매크로:**

```rust
// Rust에서: html
// JSON으로: Html로 변환
#[serde(rename = "Html")]
html: String,

// 예: JSON 직렬화 후
// {"to": "...", "Html": "...", "Subject": "..."}
```

#### 직렬화 과정

```rust
let request = SendEmailRequest {
    to: "john@example.com".to_string(),
    subject: "Confirm your email".to_string(),
    html: "<h1>Welcome John!</h1>".to_string(),
};

// Serde가 자동으로 JSON으로 변환
let json = serde_json::to_string(&request)?;
// {"to":"john@example.com","Html":"<h1>Welcome John!</h1>","Subject":"Confirm your email"}
```

### 1.5 send_email() 메서드 상세 분석

#### 전체 메서드

```rust
pub async fn send_email(
    &self,
    recipient: &str,
    subject: &str,
    html_content: &str,
) -> Result<(), String> {
    // 1. URL 구성
    let url = format!("{}/email", self.base_url);

    // 2. 요청 객체 생성
    let request = SendEmailRequest {
        to: recipient.to_string(),
        subject: subject.to_string(),
        html: html_content.to_string(),
    };

    // 3. HTTP 요청
    self.http_client
        .post(&url)
        .json(&request)
        .send()
        .await
        .map_err(|e| {
            tracing::error!("Failed to send email: {}", e);
            format!("Failed to send email: {}", e)
        })?
        .error_for_status()
        .map_err(|e| {
            tracing::error!("Email service returned error: {}", e);
            format!("Email service error: {}", e)
        })?;

    Ok(())
}
```

#### 단계별 분석

**1단계: URL 구성**

```rust
let url = format!("{}/email", self.base_url);
// self.base_url = "http://localhost:3030"
// url = "http://localhost:3030/email"
```

**2단계: 요청 객체 생성**

```rust
let request = SendEmailRequest {
    to: recipient.to_string(),
    subject: subject.to_string(),
    html: html_content.to_string(),
};
```

**3단계: HTTP POST 요청**

```rust
self.http_client
    .post(&url)                    // POST 메서드 설정
    .json(&request)                // JSON 본문 설정 (자동 직렬화)
    .send()                        // 비동기 요청 전송
    .await                         // 응답 대기
    .map_err(|e| { ... })?         // 연결 오류 처리
    .error_for_status()            // HTTP 상태 코드 확인
    .map_err(|e| { ... })?         // 상태 오류 처리
```

#### map_err와 ? 연산자

```rust
// map_err: 에러 변환
.map_err(|e| {
    // reqwest::Error → String으로 변환
    tracing::error!("Failed to send email: {}", e);
    format!("Failed to send email: {}", e)
})?

// ? 연산자: 에러 전파
// Result가 Err면 함수 즉시 반환
// Result가 Ok면 값 추출하고 계속
```

**흐름 다이어그램:**

```
self.http_client.post(&url).json(&request).send()
            │
            ├─ Success: Response
            │   ├─ .await (비동기 대기)
            │   ├─ .map_err(...) (에러 변환)
            │   ├─ ? (Ok면 계속, Err면 반환)
            │   ├─ .error_for_status() (상태 확인)
            │   │   ├─ 2xx: Ok(Response)
            │   │   └─ 4xx/5xx: Err(reqwest::Error)
            │   ├─ .map_err(...) (에러 변환)
            │   ├─ ? (최종 체크)
            │   └─ Ok(())로 반환
            │
            └─ Failure: reqwest::Error
                └─ .await에서 Err 반환
                   └─ .map_err(...) (에러 변환)
                   └─ ? 연산자로 함수 반환
```

#### 에러 처리 상세

```rust
// 연결 오류 (네트워크 문제)
.map_err(|e| {
    tracing::error!("Failed to send email: {}", e);
    format!("Failed to send email: {}", e)
})?

// 예: "error sending request for url..."

// HTTP 상태 오류 (4xx, 5xx)
.error_for_status()
.map_err(|e| {
    tracing::error!("Email service returned error: {}", e);
    format!("Email service error: {}", e)
})?

// 예: "HTTP status 500 Internal Server Error"
```

### 1.6 단위 테스트 분석

#### 테스트 1: 유효한 이메일

```rust
#[test]
fn test_confirmed_subscriber_parse_valid_email() {
    let email = "test@example.com".to_string();
    let subscriber = ConfirmedSubscriber::parse(email);
    assert!(subscriber.is_ok());
}
```

**실행 순서:**
1. 유효한 이메일 생성
2. parse() 호출
3. Ok인지 확인

**통과 조건:**
- 이메일이 RFC 5322 형식을 만족
- is_valid_email()이 Ok 반환

#### 테스트 2: 유효하지 않은 이메일

```rust
#[test]
fn test_confirmed_subscriber_parse_invalid_email() {
    let email = "invalid-email".to_string();
    let subscriber = ConfirmedSubscriber::parse(email);
    assert!(subscriber.is_err());
}
```

**실행 순서:**
1. 유효하지 않은 이메일 생성
2. parse() 호출
3. Err인지 확인

**통과 조건:**
- 이메일이 RFC 5322 형식을 만족하지 않음
- is_valid_email()이 Err 반환

---

## 2. 확인 토큰 상세 설명

### 파일 위치
```
src/confirmation_token.rs
```

### 2.1 구조체 분석

#### 전체 구조

```rust
#[derive(Clone, Debug)]
pub struct ConfirmationToken {
    token: String,
    subscriber_id: Uuid,
    created_at: chrono::DateTime<chrono::Utc>,
    expires_at: chrono::DateTime<chrono::Utc>,
}
```

**필드 설명:**

| 필드 | 타입 | 목적 | 예시 |
|------|------|------|------|
| `token` | String | 고유 토큰 | `550e8400-e29b-41d4-a716-446655440000` |
| `subscriber_id` | Uuid | 구독자 ID | `f47ac10b-58cc-4372-a567-0e02b2c3d479` |
| `created_at` | DateTime<Utc> | 생성 시간 | `2025-11-21 10:30:00 UTC` |
| `expires_at` | DateTime<Utc> | 만료 시간 | `2025-11-22 10:30:00 UTC` |

**매크로 설명:**

```rust
#[derive(Clone, Debug)]
// Clone: 토큰 복제 가능
// Debug: println!("{:?}")로 출력 가능
```

### 2.2 new() 메서드 상세 분석

#### 전체 메서드

```rust
pub fn new(subscriber_id: Uuid) -> Self {
    let token = Uuid::new_v4().to_string();
    let created_at = Utc::now();
    let expires_at = created_at + Duration::days(1);

    Self {
        token,
        subscriber_id,
        created_at,
        expires_at,
    }
}
```

#### 단계별 분석

**1단계: 토큰 생성**

```rust
let token = Uuid::new_v4().to_string();
```

```
Uuid::new_v4()
  │
  ├─ UUID v4 (무작위) 생성
  │  예: 550e8400-e29b-41d4-a716-446655440000
  │  (128비트 무작위 데이터)
  │
  └─ .to_string()
     │
     └─ String으로 변환
        예: "550e8400-e29b-41d4-a716-446655440000"
```

**UUID v4의 특징:**
- 128비트 (32개 16진수 문자)
- 무작위 생성 (예측 불가능)
- 충돌 확률 매우 낮음 (약 1조 개 중 1개 정도)
- 분산 시스템에서 안전 (중앙 레지스트리 불필요)

**2단계: 생성 시간 기록**

```rust
let created_at = Utc::now();
```

```
Utc::now()
  │
  └─ 현재 UTC 시간 반환
     예: 2025-11-21T10:30:00Z
```

**3단계: 만료 시간 계산**

```rust
let expires_at = created_at + Duration::days(1);
```

```
created_at = 2025-11-21T10:30:00Z
Duration::days(1) = 24시간
expires_at = 2025-11-22T10:30:00Z
```

**시간대 선택 (UTC):**

```rust
// ✅ 좋은 설계: UTC 사용
let created_at = Utc::now();

// ❌ 나쁜 설계: 로컬 시간 사용
let created_at = Local::now();  // 시간대 문제 발생
```

**이유:**
- 서버가 여러 지역에 분산되어도 일관성 유지
- 데이터베이스와 일치
- 비교 연산 정확

### 2.3 Getter 메서드 분석

#### token() 메서드

```rust
pub fn token(&self) -> &str {
    &self.token
}
```

**왜 &str을 반환하는가?**

```rust
// ❌ 비효율: String 복제
pub fn token(&self) -> String {
    self.token.clone()
}

// ✅ 효율: 참조만 전달
pub fn token(&self) -> &str {
    &self.token
}

// 장점:
// - 복제 없음 (메모리 절약)
// - 성능 향상
// - Rust 권장 패턴
```

#### subscriber_id() 메서드

```rust
pub fn subscriber_id(&self) -> Uuid {
    self.subscriber_id
}
```

**왜 Uuid를 직접 반환하는가?**

```rust
// Uuid는 Copy 타입
// struct Uuid([u8; 16])는 작은 크기 (16바이트)
// 복제가 빠르므로 직접 반환
// String(heap 할당)과 달리 스택 할당

let id1 = token.subscriber_id();
let id2 = token.subscriber_id();  // 복제 비용 낮음
```

#### created_at() / expires_at() 메서드

```rust
pub fn created_at(&self) -> chrono::DateTime<chrono::Utc> {
    self.created_at
}

pub fn expires_at(&self) -> chrono::DateTime<chrono::Utc> {
    self.expires_at
}
```

**DateTime<Utc>도 Copy 타입**

```rust
// DateTime<Utc>도 스택 할당 (작은 크기)
// 복제 비용 낮음
// 직접 반환 (참조 불필요)
```

### 2.4 is_expired() 메서드 상세 분석

#### 메서드 정의

```rust
pub fn is_expired(&self) -> bool {
    Utc::now() > self.expires_at
}
```

#### 동작 원리

```
현재 시간 vs 만료 시간
     │
     ├─ 현재 > 만료: true (만료됨)
     │  예: 2025-11-22 11:00 > 2025-11-22 10:30
     │
     └─ 현재 ≤ 만료: false (유효)
        예: 2025-11-22 10:30 ≤ 2025-11-22 10:30
```

#### 사용 예

```rust
// 토큰 생성
let token = ConfirmationToken::new(subscriber_id);

// 즉시 확인
assert!(!token.is_expired());  // false (유효)

// 24시간 후 확인 (시뮬레이션)
// ... 시간 경과 ...
assert!(token.is_expired());   // true (만료됨)
```

#### 데이터베이스 쿼리에 사용

```rust
// is_expired() 메서드 대신 SQL WHERE 절 사용
SELECT subscriber_id FROM subscription_tokens
WHERE subscription_token = $1
AND expires_at > NOW();  // 만료 시간 확인
```

**이유:**
- 데이터베이스에서 이미 UTC 시간 저장
- SQL의 NOW()와 Rust의 Utc::now() 일치
- 데이터베이스가 필터링 (성능 향상)

### 2.5 단위 테스트 분석

#### 테스트 1: 토큰 생성

```rust
#[test]
fn test_confirmation_token_creation() {
    let subscriber_id = Uuid::new_v4();
    let token = ConfirmationToken::new(subscriber_id);

    assert_eq!(token.subscriber_id(), subscriber_id);
    assert!(!token.is_expired());
}
```

**검증 항목:**
1. 토큰이 정상 생성됨
2. subscriber_id가 올바르게 저장됨
3. 생성 직후 만료되지 않음

#### 테스트 2: 만료 여부 확인

```rust
#[test]
fn test_confirmation_token_not_immediately_expired() {
    let subscriber_id = Uuid::new_v4();
    let token = ConfirmationToken::new(subscriber_id);

    assert!(!token.is_expired());
}
```

**검증 항목:**
- 생성된 토큰이 즉시 만료되지 않음
- is_expired() 메서드가 정상 동작

---

## 3. 확인 엔드포인트 상세 설명

### 파일 위치
```
src/routes/confirmation.rs
```

### 3.1 라우트 핸들러 분석

#### 전체 핸들러

```rust
pub async fn confirm_subscription(
    query: web::Query<ConfirmationQuery>,
    pool: web::Data<PgPool>,
) -> HttpResponse {
    let token = &query.token;

    match get_subscriber_id_from_token(pool.get_ref(), token).await {
        Ok(Some(subscriber_id)) => {
            match update_subscription_status(pool.get_ref(), &subscriber_id, "confirmed").await {
                Ok(_) => {
                    tracing::info!(
                        subscriber_id = %subscriber_id,
                        "Subscription confirmed successfully"
                    );
                    HttpResponse::Ok().json(serde_json::json!({
                        "message": "Thank you for confirming your subscription!"
                    }))
                }
                Err(e) => { /* 에러 처리 */ }
            }
        }
        Ok(None) => { /* 토큰 없음 */ }
        Err(e) => { /* DB 에러 */ }
    }
}
```

#### 함수 서명 분석

```rust
pub async fn confirm_subscription(
    query: web::Query<ConfirmationQuery>,  // URL 쿼리 파라미터
    pool: web::Data<PgPool>,               // 데이터베이스 연결 풀
) -> HttpResponse {                        // HTTP 응답
```

**파라미터:**

| 파라미터 | 타입 | 출처 | 예시 |
|---------|------|------|------|
| `query` | `web::Query<ConfirmationQuery>` | URL | `?token=abc123` |
| `pool` | `web::Data<PgPool>` | 의존성 주입 | Actix-web |

### 3.2 쿼리 구조체 분석

#### 정의

```rust
#[derive(Deserialize)]
pub struct ConfirmationQuery {
    token: String,
}
```

#### Serde 역직렬화

```
URL: /subscriptions/confirm?token=550e8400-e29b-41d4-a716-446655440000
  │
  ├─ Actix-web이 URL 파싱
  │
  ├─ web::Query<ConfirmationQuery>가 처리
  │
  ├─ Serde #[derive(Deserialize)]가 역직렬화
  │  token 필드 ← "550e8400-e29b-41d4-a716-446655440000" (문자열)
  │
  └─ ConfirmationQuery { token: "550e8400..." }
```

#### 사용 예

```rust
let token = &query.token;
// token = "550e8400-e29b-41d4-a716-446655440000"
```

### 3.3 데이터베이스 조회 함수 분석

#### get_subscriber_id_from_token()

```rust
async fn get_subscriber_id_from_token(
    pool: &PgPool,
    token: &str,
) -> Result<Option<String>, sqlx::Error> {
    let result = sqlx::query_as::<_, (String,)>(
        r#"
        SELECT subscriber_id
        FROM subscription_tokens
        WHERE subscription_token = $1
        AND expires_at > NOW()
        "#,
    )
    .bind(token)
    .fetch_optional(pool)
    .await?;

    Ok(result.map(|(id,)| id))
}
```

#### 쿼리 분석

```sql
SELECT subscriber_id           -- 1. 선택: subscriber_id 컬럼
FROM subscription_tokens       -- 2. 테이블: subscription_tokens
WHERE subscription_token = $1  -- 3. 조건: 토큰 일치
AND expires_at > NOW()         -- 4. 조건: 아직 만료 안됨
```

**동작:**

```
입력 토큰: "550e8400-e29b-41d4-a716-446655440000"
  │
  ├─ 데이터베이스 쿼리 실행
  │  SELECT subscriber_id FROM subscription_tokens
  │  WHERE subscription_token = '550e8400-e29b-41d4-a716-446655440000'
  │  AND expires_at > NOW()
  │
  ├─ 경우 1: 토큰 존재 & 유효
  │  └─ Some("f47ac10b-58cc-4372-a567-0e02b2c3d479")
  │
  └─ 경우 2: 토큰 없음 또는 만료
     └─ None
```

#### sqlx::query_as 분석

```rust
sqlx::query_as::<_, (String,)>(sql)
         │         │  └─ 반환 타입: (String,) (튜플)
         │         └─ 데이터베이스 타입 (생략 시 자동 추론)
         └─ SQL → Rust 구조체로 변환
```

#### bind() 메서드

```rust
.bind(token)
```

```
token = "550e8400-e29b-41d4-a716-446655440000"
  │
  ├─ $1 위치에 바인드
  │  WHERE subscription_token = $1
  │                             ↑
  │                             token 값 여기에 삽입
  │
  └─ SQL 인젝션 방지
     데이터베이스가 안전하게 처리
```

#### fetch_optional() 메서드

```rust
.fetch_optional(pool)
```

```
Result<Option<(String,)>, sqlx::Error>
  │
  ├─ Ok(Some((id,)))     # 결과 있음
  ├─ Ok(None)            # 결과 없음
  └─ Err(sqlx::Error)    # DB 오류
```

#### await와 ? 연산자

```rust
.await?

.await: 비동기 완료 대기
?:      Result<T, E> 처리
        ├─ Ok(v): v 추출, 계속
        └─ Err(e): 함수 즉시 반환
```

#### 결과 변환

```rust
Ok(result.map(|(id,)| id))

result = Some((id,))
  │
  ├─ .map(): Some 안의 값 변환
  │  |(id,)| ← 튜플에서 id 추출
  │  id     ← id 반환
  │
  └─ Some(id)로 변환
```

### 3.4 상태 업데이트 함수 분석

#### update_subscription_status()

```rust
async fn update_subscription_status(
    pool: &PgPool,
    subscriber_id: &str,
    status: &str,
) -> Result<(), sqlx::Error> {
    // 1. 상태 업데이트
    sqlx::query(
        r#"
        UPDATE subscriptions
        SET status = $1
        WHERE id = $2
        "#,
    )
    .bind(status)
    .bind(subscriber_id)
    .execute(pool)
    .await?;

    // 2. 토큰 삭제
    sqlx::query(
        r#"
        DELETE FROM subscription_tokens
        WHERE subscriber_id = $1
        "#,
    )
    .bind(subscriber_id)
    .execute(pool)
    .await?;

    Ok(())
}
```

#### 1단계: 상태 업데이트

```sql
UPDATE subscriptions
SET status = $1
WHERE id = $2
```

```
$1: 'confirmed' (새 상태)
$2: subscriber_id (구독자 ID)

예:
UPDATE subscriptions
SET status = 'confirmed'
WHERE id = 'f47ac10b-58cc-4372-a567-0e02b2c3d479'
```

**실행 후:**
- `subscriptions` 테이블에서 해당 구독자의 `status` = 'confirmed'

#### 2단계: 토큰 삭제

```sql
DELETE FROM subscription_tokens
WHERE subscriber_id = $1
```

```
$1: subscriber_id (구독자 ID)

예:
DELETE FROM subscription_tokens
WHERE subscriber_id = 'f47ac10b-58cc-4372-a567-0e02b2c3d479'
```

**실행 후:**
- `subscription_tokens` 테이블에서 해당 토큰 삭제
- 이 토큰으로는 다시 확인 불가능 (일회용)

#### execute() 메서드

```rust
.execute(pool)
```

```
쿼리 실행 후 영향받은 행 수 반환
Result<sqlx::sqlite::SqliteQueryResult, sqlx::Error>
  │
  ├─ Ok(result): result.rows_affected() 사용 가능
  └─ Err(error): 실행 오류
```

### 3.5 에러 처리 분석

#### 3단계 매칭

```rust
match get_subscriber_id_from_token(pool.get_ref(), token).await {
    Ok(Some(subscriber_id)) => { ... },      // 토큰 유효
    Ok(None) => { ... },                      // 토큰 없음/만료
    Err(e) => { ... },                        // DB 오류
}
```

#### 케이스 1: 토큰 유효

```rust
Ok(Some(subscriber_id)) => {
    match update_subscription_status(...).await {
        Ok(_) => {
            tracing::info!(
                subscriber_id = %subscriber_id,
                "Subscription confirmed successfully"
            );
            HttpResponse::Ok().json(serde_json::json!({
                "message": "Thank you for confirming your subscription!"
            }))
        }
        Err(e) => {
            tracing::error!(
                subscriber_id = %subscriber_id,
                error = %e,
                "Failed to update subscription status"
            );
            HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Failed to confirm subscription"
            }))
        }
    }
}
```

**응답:**
```json
{
  "message": "Thank you for confirming your subscription!"
}
```

**HTTP 상태:** 200 OK

#### 케이스 2: 토큰 없음/만료

```rust
Ok(None) => {
    tracing::warn!(
        token = %token,
        "Invalid or expired confirmation token"
    );
    HttpResponse::BadRequest().json(serde_json::json!({
        "error": "Invalid or expired confirmation token"
    }))
}
```

**응답:**
```json
{
  "error": "Invalid or expired confirmation token"
}
```

**HTTP 상태:** 400 Bad Request

#### 케이스 3: 데이터베이스 오류

```rust
Err(e) => {
    tracing::error!(
        error = %e,
        "Database error while confirming subscription"
    );
    HttpResponse::InternalServerError().json(serde_json::json!({
        "error": "Failed to confirm subscription"
    }))
}
```

**응답:**
```json
{
  "error": "Failed to confirm subscription"
}
```

**HTTP 상태:** 500 Internal Server Error

---

## 4. 구독 엔드포인트 통합

### 파일 위치
```
src/routes/subscriptions.rs
```

### 4.1 함수 서명 변경

#### 변경 전

```rust
pub async fn subscribe(
    form: web::Form<FormData>,
    pool: web::Data<PgPool>,
) -> HttpResponse {
```

#### 변경 후

```rust
pub async fn subscribe(
    form: web::Form<FormData>,
    pool: web::Data<PgPool>,
    email_client: web::Data<EmailClient>,  // 🆕 추가
) -> HttpResponse {
```

**추가 파라미터:**
- `email_client`: Actix-web이 의존성 주입
- `web::Data<T>`: Arc 기반 공유 데이터
- 여러 핸들러가 동시에 사용 가능

### 4.2 INSERT 쿼리 변경

#### 변경 전

```rust
sqlx::query(
    "INSERT INTO subscriptions (id, email, name, subscribed_at) VALUES ($1, $2, $3, $4)"
)
.bind(subscriber_id)
.bind(&email)
.bind(&name)
.bind(Utc::now())
.execute(pool.get_ref())
.await
```

#### 변경 후

```rust
sqlx::query(
    "INSERT INTO subscriptions (id, email, name, subscribed_at, status) VALUES ($1, $2, $3, $4, $5)"
)
.bind(subscriber_id)
.bind(&email)
.bind(&name)
.bind(Utc::now())
.bind("pending")  // 🆕 상태 추가
.execute(pool.get_ref())
.await
```

**변경사항:**
- `status` 필드 추가
- VALUES에 $5 추가
- "pending" 상태로 초기화

### 4.3 토큰 생성 및 저장

#### 토큰 생성

```rust
let confirmation_token = ConfirmationToken::new(subscriber_id);
```

```
subscriber_id = 'f47ac10b-58cc-4372-a567-0e02b2c3d479'
  │
  └─ ConfirmationToken::new()
     │
     ├─ token = Uuid::new_v4().to_string()
     │          = "550e8400-e29b-41d4-a716-446655440000"
     │
     ├─ subscriber_id = 'f47ac10b-58cc-4372-a567-0e02b2c3d479'
     │
     ├─ created_at = Utc::now()
     │               = 2025-11-21T10:30:00Z
     │
     └─ expires_at = Utc::now() + 24h
                    = 2025-11-22T10:30:00Z

결과:
ConfirmationToken {
    token: "550e8400-e29b-41d4-a716-446655440000",
    subscriber_id: Uuid(...),
    created_at: 2025-11-21T10:30:00Z,
    expires_at: 2025-11-22T10:30:00Z,
}
```

#### 토큰 저장

```rust
if let Err(e) = sqlx::query(
    r#"
    INSERT INTO subscription_tokens
    (subscription_token, subscriber_id, created_at, expires_at)
    VALUES ($1, $2, $3, $4)
    "#
)
.bind(confirmation_token.token())
.bind(subscriber_id.to_string())
.bind(confirmation_token.created_at())
.bind(confirmation_token.expires_at())
.execute(pool.get_ref())
.await
{
    tracing::error!(
        subscriber_id = %subscriber_id,
        error = %e,
        "Failed to save confirmation token"
    );
    return HttpResponse::InternalServerError().finish();
}
```

**바인드 값:**
- $1: "550e8400-e29b-41d4-a716-446655440000" (토큰)
- $2: "f47ac10b-58cc-4372-a567-0e02b2c3d479" (구독자 ID)
- $3: 2025-11-21T10:30:00Z (생성 시간)
- $4: 2025-11-22T10:30:00Z (만료 시간)

**에러 처리:**
- 실패 시 500 Internal Server Error 반환
- 함수 즉시 종료

### 4.4 이메일 생성

#### 확인 링크 생성

```rust
let confirmation_link = format!(
    "http://localhost:8000/subscriptions/confirm?token={}",
    confirmation_token.token()
);
```

```
confirmation_token.token() = "550e8400-e29b-41d4-a716-446655440000"
  │
  └─ format! 매크로
     │
     └─ "http://localhost:8000/subscriptions/confirm?token=550e8400-e29b-41d4-a716-446655440000"
```

#### HTML 콘텐츠 생성

```rust
let html_content = format!(
    r#"
    <h1>Welcome {}!</h1>
    <p>Please confirm your email subscription by clicking the link below:</p>
    <a href="{}">Confirm Subscription</a>
    <p>This link will expire in 24 hours.</p>
    "#,
    name, confirmation_link
);
```

```
name = "John Doe"
confirmation_link = "http://localhost:8000/subscriptions/confirm?token=550e8400..."
  │
  └─ format! 매크로
     │
     └─ HTML 문자열:
        <h1>Welcome John Doe!</h1>
        <p>Please confirm your email subscription...</p>
        <a href="http://localhost:8000/subscriptions/confirm?token=550e8400-e29b-41d4-a716-446655440000">
          Confirm Subscription
        </a>
        <p>This link will expire in 24 hours.</p>
```

### 4.5 이메일 전송

#### 헬퍼 함수

```rust
async fn send_confirmation_email(
    email_client: &EmailClient,
    recipient_email: &str,
    html_content: &str,
) -> Result<(), String> {
    email_client
        .send_email(
            recipient_email,
            "Please confirm your subscription",
            html_content,
        )
        .await
}
```

**파라미터:**
- `email_client`: 이메일 클라이언트
- `recipient_email`: 수신자 이메일
- `html_content`: HTML 콘텐츠

#### 호출

```rust
if let Err(e) = send_confirmation_email(
    email_client.get_ref(),
    &email,
    &html_content,
)
.await
{
    tracing::error!(
        subscriber_id = %subscriber_id,
        error = %e,
        "Failed to send confirmation email"
    );
    return HttpResponse::InternalServerError().finish();
}
```

**get_ref() 메서드:**
```rust
email_client: web::Data<EmailClient>
  │
  ├─ .get_ref()
  │  │
  │  └─ &EmailClient 추출
  │     (Arc 내부의 참조 얻기)
```

**에러 처리:**
- 실패 시 500 Internal Server Error 반환

---

## 5. 데이터베이스 스키마

### 5.1 subscriptions 테이블

#### 생성 SQL

```sql
CREATE TABLE subscriptions(
    id uuid NOT NULL,
    email TEXT NOT NULL UNIQUE,
    name TEXT NOT NULL,
    subscribed_at timestamptz NOT NULL,
    PRIMARY KEY (id)
);

-- 이후 추가:
ALTER TABLE subscriptions
ADD COLUMN status VARCHAR(20) NOT NULL DEFAULT 'pending';
```

#### 테이블 구조

| 컬럼 | 타입 | 제약 | 설명 |
|------|------|------|------|
| id | uuid | PK | 구독자 고유 ID |
| email | TEXT | UNIQUE | 이메일 (중복 불가) |
| name | TEXT | NOT NULL | 구독자 이름 |
| subscribed_at | timestamptz | NOT NULL | 구독 시간 |
| status | VARCHAR(20) | NOT NULL, DEFAULT 'pending' | 상태 |

#### 상태 값

```
상태: pending
설명: 이메일 확인 대기
예: 구독 직후

상태: confirmed
설명: 이메일 확인 완료
예: 확인 링크 클릭 후
```

#### 예시 데이터

```sql
INSERT INTO subscriptions VALUES (
    'f47ac10b-58cc-4372-a567-0e02b2c3d479',
    'john@example.com',
    'John Doe',
    '2025-11-21 10:30:00+00',
    'pending'
);
```

### 5.2 subscription_tokens 테이블

#### 생성 SQL

```sql
CREATE TABLE subscription_tokens(
    subscription_token TEXT NOT NULL,
    subscriber_id uuid NOT NULL REFERENCES subscriptions (id) ON DELETE CASCADE,
    created_at timestamptz NOT NULL,
    expires_at timestamptz NOT NULL,
    PRIMARY KEY (subscription_token)
);

CREATE INDEX idx_subscription_tokens_subscriber_id
ON subscription_tokens(subscriber_id);
```

#### 테이블 구조

| 컬럼 | 타입 | 제약 | 설명 |
|------|------|------|------|
| subscription_token | TEXT | PK | 고유 토큰 |
| subscriber_id | uuid | FK | 구독자 참조 |
| created_at | timestamptz | NOT NULL | 생성 시간 |
| expires_at | timestamptz | NOT NULL | 만료 시간 |

#### 외래 키 설정

```sql
REFERENCES subscriptions (id) ON DELETE CASCADE
  │                       │      │
  │                       │      └─ 구독자 삭제 시 토큰도 자동 삭제
  │                       └─ subscriptions 테이블의 id 참조
  └─ 외래 키 제약
```

#### 인덱싱

```sql
CREATE INDEX idx_subscription_tokens_subscriber_id
ON subscription_tokens(subscriber_id);

-- 빠른 조회:
SELECT * FROM subscription_tokens
WHERE subscriber_id = $1;  -- 인덱스 사용 (O(log n))
```

#### 예시 데이터

```sql
INSERT INTO subscription_tokens VALUES (
    '550e8400-e29b-41d4-a716-446655440000',
    'f47ac10b-58cc-4372-a567-0e02b2c3d479',
    '2025-11-21 10:30:00+00',
    '2025-11-22 10:30:00+00'
);
```

### 5.3 관계도

```
subscriptions (1)
    │
    │ (1 : N)
    │ FOREIGN KEY (subscriber_id) REFERENCES subscriptions (id)
    │
subscription_tokens (N)
```

```sql
-- 구독자 삭제 시
DELETE FROM subscriptions
WHERE id = 'f47ac10b-58cc-4372-a567-0e02b2c3d479';

-- 자동으로 관련 토큰도 삭제
-- (ON DELETE CASCADE)
DELETE FROM subscription_tokens
WHERE subscriber_id = 'f47ac10b-58cc-4372-a567-0e02b2c3d479';
```

---

## 요약

이 문서에서 다룬 주요 내용:

1. **이메일 클라이언트**: HTTP를 통한 비동기 이메일 전송
2. **확인 토큰**: UUID v4 기반의 24시간 유효 토큰
3. **확인 엔드포인트**: 토큰 검증 및 상태 업데이트
4. **구독 통합**: 구독 시 자동으로 이메일 전송
5. **데이터베이스 스키마**: 관계형 테이블 설계

각 컴포넌트는 서로 독립적이면서도 조화롭게 작동하여 완전한 이메일 확인 서비스를 제공합니다.
