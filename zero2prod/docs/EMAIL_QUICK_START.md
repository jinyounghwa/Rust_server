# 이메일 확인 서비스 빠른 시작 가이드

## 5분 안에 시작하기

### 1단계: 데이터베이스 마이그레이션
```bash
sqlx migrate run
```

### 2단계: 애플리케이션 실행
```bash
RUST_LOG=debug cargo run
```

### 3단계: 구독 요청
```bash
curl -X POST http://localhost:8000/subscriptions \
  -H "Content-Type: application/x-www-form-urlencoded" \
  -d "name=Test User&email=test@example.com"
```

**응답:** `200 OK`

### 4단계: 데이터베이스에서 토큰 확인
```bash
psql -U postgres -d zero2prod \
  -c "SELECT subscription_token FROM subscription_tokens LIMIT 1;"
```

### 5단계: 확인 링크 클릭
```bash
curl "http://localhost:8000/subscriptions/confirm?token=<TOKEN_HERE>"
```

**응답:**
```json
{
  "message": "Thank you for confirming your subscription!"
}
```

### 6단계: 구독 상태 확인
```bash
psql -U postgres -d zero2prod \
  -c "SELECT email, status FROM subscriptions WHERE email='test@example.com';"
```

**출력:**
```
     email      | status
----------------|---------
test@example.com| confirmed
```

---

## 파일 구조

```
src/
├── email_client.rs           # 이메일 클라이언트 구현
├── confirmation_token.rs     # 확인 토큰 생성 및 검증
└── routes/
    ├── subscriptions.rs      # 구독 엔드포인트 (이메일 전송 포함)
    └── confirmation.rs       # 확인 엔드포인트

migrations/
├── 20231105000001_create_subscriptions_table.up.sql
└── 20231105000002_create_subscription_tokens_table.up.sql

docs/
├── EMAIL_CONFIRMATION_SERVICE.md  # 상세 설명서
├── SETUP_GUIDE.md                 # 설정 가이드
└── EMAIL_QUICK_START.md           # 이 파일
```

---

## 핵심 컴포넌트

### EmailClient
```rust
pub struct EmailClient {
    pub async fn send_email(
        &self,
        recipient: &str,
        subject: &str,
        html_content: &str,
    ) -> Result<(), String>
}
```

### ConfirmationToken
```rust
pub struct ConfirmationToken {
    // UUID 기반 토큰 (자동 생성)
    // 24시간 유효
    // 확인 후 자동 삭제
}
```

---

## 워크플로우 다이어그램

```
┌─────────────────────────────────────────────────────────────┐
│                    사용자                                    │
└───────────────────────┬─────────────────────────────────────┘
                        │
                        │ 1. 이메일 & 이름 입력
                        ▼
    ┌────────────────────────────────────────┐
    │  POST /subscriptions                   │
    │  {name, email}                         │
    └────────────┬─────────────────────────────┘
                 │
                 ▼
    ┌────────────────────────────────────────┐
    │  1. 입력값 검증                        │
    │  2. subscriptions 테이블에 저장        │
    │     (status='pending')                 │
    │  3. 확인 토큰 생성                     │
    │  4. subscription_tokens 저장           │
    │  5. 이메일 전송                        │
    │  6. 응답: 200 OK                       │
    └────────────┬────────────────────────────┘
                 │
                 ▼
    ┌────────────────────────────────────────┐
    │  📧 이메일 수신                         │
    │  "Confirm Subscription 링크 포함"      │
    └────────────┬────────────────────────────┘
                 │
                 │ 2. 링크 클릭
                 ▼
    ┌────────────────────────────────────────┐
    │  GET /subscriptions/confirm?token=...  │
    └────────────┬─────────────────────────────┘
                 │
                 ▼
    ┌────────────────────────────────────────┐
    │  1. 토큰 유효성 검증                   │
    │  2. 만료 시간 확인                     │
    │  3. status='confirmed' 업데이트        │
    │  4. 토큰 삭제                          │
    │  5. 응답: 200 OK (성공 메시지)        │
    └────────────┬────────────────────────────┘
                 │
                 ▼
    ┌────────────────────────────────────────┐
    │  ✅ 구독 확인 완료!                    │
    │  status='confirmed'                    │
    └────────────────────────────────────────┘
```

---

## 테스트 시나리오

### 시나리오 1: 정상 흐름
```bash
# 구독
curl -X POST http://localhost:8000/subscriptions \
  -d "name=Alice&email=alice@example.com"

# 토큰 조회
TOKEN=$(psql -U postgres -d zero2prod -t \
  -c "SELECT subscription_token FROM subscription_tokens LIMIT 1;")

# 확인
curl "http://localhost:8000/subscriptions/confirm?token=$TOKEN"
```

### 시나리오 2: 잘못된 이메일
```bash
curl -X POST http://localhost:8000/subscriptions \
  -d "name=Bob&email=invalid-email"
# 응답: 400 Bad Request
```

### 시나리오 3: 중복 이메일
```bash
curl -X POST http://localhost:8000/subscriptions \
  -d "name=Charlie&email=charlie@example.com"

curl -X POST http://localhost:8000/subscriptions \
  -d "name=Charlie&email=charlie@example.com"
# 두 번째 응답: 409 Conflict
```

### 시나리오 4: 잘못된 토큰
```bash
curl "http://localhost:8000/subscriptions/confirm?token=wrong-token"
# 응답: 400 Bad Request
# {"error": "Invalid or expired confirmation token"}
```

---

## 데이터베이스 조회

### 모든 구독자 확인
```sql
SELECT email, status, subscribed_at FROM subscriptions;
```

### 대기 중인 구독자
```sql
SELECT email FROM subscriptions WHERE status='pending';
```

### 확인된 구독자
```sql
SELECT email FROM subscriptions WHERE status='confirmed';
```

### 유효한 토큰
```sql
SELECT subscription_token, expires_at FROM subscription_tokens
WHERE expires_at > NOW();
```

### 만료된 토큰
```sql
SELECT subscription_token, expires_at FROM subscription_tokens
WHERE expires_at <= NOW();
```

---

## 환경 변수

| 변수 | 설명 | 기본값 |
|------|------|--------|
| `DATABASE_URL` | PostgreSQL 연결 문자열 | 필수 |
| `RUST_LOG` | 로그 레벨 | `info` |
| `SERVER_PORT` | 서버 포트 | `8000` |
| `SERVER_HOST` | 서버 호스트 | `127.0.0.1` |

---

## 문제 해결

### "Failed to save subscriber to database"
- PostgreSQL 연결 확인: `psql postgresql://user:pass@localhost/zero2prod`
- 마이그레이션 실행 확인: `sqlx migrate info`

### "Invalid or expired confirmation token"
- 토큰이 유효한지 확인: `SELECT * FROM subscription_tokens WHERE subscription_token='...'`
- 만료 시간 확인: `expires_at > NOW()`

### "Duplicate email subscription attempt"
- 이미 등록된 이메일입니다
- 다른 이메일로 시도하거나, 기존 구독을 확인하세요

---

## 다음 단계

1. **프로덕션 이메일 서비스 통합**: SendGrid, AWS SES 등
2. **UI 개발**: 웹 폼으로 구독 기능 노출
3. **모니터링**: 이메일 전송 실패 추적
4. **메일링 큐**: 이메일 전송 실패 재시도 로직
5. **분석**: 구독자 행동 분석

자세한 내용은 `docs/EMAIL_CONFIRMATION_SERVICE.md`를 참고하세요!
