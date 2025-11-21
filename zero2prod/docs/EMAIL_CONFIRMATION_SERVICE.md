# 이메일 확인 서비스 가이드

## 개요
이 서비스는 가상 이메일 클라이언트를 사용하여 이메일 구독 확인 기능을 구현합니다.

## 시스템 플로우

```
1. 사용자가 이메일 구독 요청
   ↓
2. 서버: 구독자 정보 저장 (상태: pending)
   ↓
3. 서버: 24시간 유효한 확인 토큰 생성
   ↓
4. 서버: 이메일 클라이언트를 통해 확인 링크 전송
   ↓
5. 사용자: 이메일의 확인 링크 클릭
   ↓
6. 서버: 토큰 검증 및 구독 상태를 'confirmed'로 업데이트
   ↓
7. 토큰 자동 삭제
```

## API 엔드포인트

### 1. 구독 요청
**POST** `/subscriptions`

**요청:**
```json
{
  "name": "John Doe",
  "email": "john@example.com"
}
```

**응답:**
- 성공: `200 OK`
- 이메일 형식 오류: `400 Bad Request`
- 중복 이메일: `409 Conflict`
- 서버 오류: `500 Internal Server Error`

**프로세스:**
1. 이메일과 이름 검증
2. `subscriptions` 테이블에 새 레코드 생성 (상태: pending)
3. 확인 토큰 생성
4. `subscription_tokens` 테이블에 토큰 저장
5. 이메일 클라이언트를 통해 확인 링크 전송

### 2. 구독 확인
**GET** `/subscriptions/confirm?token={token}`

**응답:**
```json
{
  "message": "Thank you for confirming your subscription!"
}
```

또는 오류 시:
```json
{
  "error": "Invalid or expired confirmation token"
}
```

**프로세스:**
1. 토큰 유효성 검증
2. 토큰 만료 시간 확인
3. 구독자 ID 조회
4. 구독 상태를 'confirmed'로 업데이트
5. 사용한 토큰 삭제

## 데이터베이스 스키마

### subscriptions 테이블
```sql
CREATE TABLE subscriptions(
    id uuid NOT NULL PRIMARY KEY,
    email TEXT NOT NULL UNIQUE,
    name TEXT NOT NULL,
    subscribed_at timestamptz NOT NULL,
    status VARCHAR(20) NOT NULL DEFAULT 'pending'
);
```

**상태 값:**
- `pending`: 이메일 확인 대기 중
- `confirmed`: 이메일 확인 완료

### subscription_tokens 테이블
```sql
CREATE TABLE subscription_tokens(
    subscription_token TEXT NOT NULL PRIMARY KEY,
    subscriber_id uuid NOT NULL REFERENCES subscriptions (id) ON DELETE CASCADE,
    created_at timestamptz NOT NULL,
    expires_at timestamptz NOT NULL
);
```

## 이메일 클라이언트 구현

### EmailClient 구조
```rust
pub struct EmailClient {
    http_client: reqwest::Client,
    base_url: String,
    sender: ConfirmedSubscriber,
}
```

### 메서드
```rust
pub async fn send_email(
    &self,
    recipient: &str,
    subject: &str,
    html_content: &str,
) -> Result<(), String>
```

## 확인 토큰 구현

### ConfirmationToken 구조
```rust
pub struct ConfirmationToken {
    token: String,                        // UUID 기반 토큰
    subscriber_id: Uuid,                  // 구독자 ID
    created_at: chrono::DateTime<Utc>,    // 생성 시간
    expires_at: chrono::DateTime<Utc>,    // 만료 시간 (24시간 후)
}
```

### 토큰 특징
- 자동 생성: UUID v4 사용
- 유효 기간: 24시간
- 일회용: 확인 후 자동 삭제

## 가상 이메일 서버 사용법

### 1. 가상 서버 구성
프로덕션 환경에서는 실제 이메일 서비스(SendGrid, AWS SES 등)를 사용하세요.
테스트 환경에서는 이 가상 클라이언트를 사용할 수 있습니다.

### 2. 이메일 전송 흐름
```rust
// src/routes/subscriptions.rs에서 사용
let html_content = format!(
    r#"
    <h1>Welcome {}!</h1>
    <p>Please confirm your email subscription by clicking the link below:</p>
    <a href="{}">Confirm Subscription</a>
    <p>This link will expire in 24 hours.</p>
    "#,
    name, confirmation_link
);

email_client.send_email(&email, subject, &html_content).await?
```

## 보안 기능

### 입력 검증
- 이메일 형식 검증 (RFC 5322)
- 이름 길이 및 문자 검증
- SQL 인젝션 방지 (parameterized queries)

### 토큰 보안
- UUID 기반 강력한 토큰
- 24시간 제한된 유효 기간
- 사용 후 자동 삭제
- 데이터베이스 무결성 제약 (FK, 인덱싱)

### 인덱싱
```sql
-- 빠른 조회
CREATE INDEX idx_subscription_tokens_subscriber_id
ON subscription_tokens(subscriber_id);

CREATE INDEX idx_subscriptions_status
ON subscriptions(status);
```

## 테스트 시나리오

### 시나리오 1: 정상 구독
```bash
curl -X POST http://localhost:8000/subscriptions \
  -d "name=John&email=john@example.com"
# 응답: 200 OK
```

### 시나리오 2: 이메일 확인
```bash
# 1. 이메일에서 받은 링크 형식
http://localhost:8000/subscriptions/confirm?token=<token>

# 응답: {"message": "Thank you for confirming your subscription!"}
```

### 시나리오 3: 잘못된 토큰
```bash
curl "http://localhost:8000/subscriptions/confirm?token=invalid-token"
# 응답: 400 Bad Request
# {"error": "Invalid or expired confirmation token"}
```

### 시나리오 4: 만료된 토큰
```bash
# 24시간 이후
curl "http://localhost:8000/subscriptions/confirm?token=<old-token>"
# 응답: 400 Bad Request
# {"error": "Invalid or expired confirmation token"}
```

## 트러블슈팅

### 이메일이 전송되지 않음
1. EmailClient 설정 확인
2. 네트워크 연결 확인
3. 로그에서 에러 메시지 확인

### 토큰이 만료됨
- 토큰 유효 기간은 24시간입니다
- 새로운 구독을 생성하여 새 토큰을 받으세요

### 중복 이메일 오류
- 같은 이메일로는 중복 구독이 불가능합니다
- 409 Conflict 응답을 받으면 이미 등록된 이메일입니다

## 프로덕션 배포 체크리스트

- [ ] 실제 이메일 서비스 통합 (SendGrid, AWS SES 등)
- [ ] 환경 변수로 이메일 설정 관리
- [ ] 이메일 템플릿 HTML/CSS 개선
- [ ] 토큰 유효 기간 조정 (필요시)
- [ ] 재시도 로직 추가
- [ ] 메일링 큐 시스템 구현
- [ ] 모니터링 및 로깅 강화
- [ ] 레이트 제한 추가
