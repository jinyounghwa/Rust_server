# 이메일 확인 서비스 설정 가이드

## 사전 요구사항

- Rust 1.70+
- PostgreSQL 12+
- sqlx-cli (마이그레이션용)

## 1. 환경 설정

### .env 파일 생성
```bash
DATABASE_URL=postgres://username:password@localhost:5432/zero2prod
RUST_LOG=debug
SERVER_PORT=8000
SERVER_HOST=127.0.0.1
```

### configuration.yaml 설정
```yaml
server:
  port: 8000
  host: 127.0.0.1

database:
  host: localhost
  port: 5432
  username: postgres
  password: password
  database_name: zero2prod
  require_ssl: false
```

## 2. 데이터베이스 설정

### PostgreSQL 데이터베이스 생성
```bash
psql -U postgres -c "CREATE DATABASE zero2prod;"
```

### 마이그레이션 실행
```bash
# sqlx-cli 설치 (필요시)
cargo install sqlx-cli --no-default-features --features postgres

# 마이그레이션 실행
sqlx migrate run
```

마이그레이션 파일:
- `migrations/20231105000001_create_subscriptions_table.up.sql`
- `migrations/20231105000002_create_subscription_tokens_table.up.sql`

## 3. 이메일 클라이언트 설정

### 테스트 환경 (가상 서버)
```rust
// src/main.rs에서
use zero2prod::email_client::{EmailClient, ConfirmedSubscriber};

let email_client = EmailClient::new(
    "http://localhost:3030".to_string(),  // 가상 이메일 서버 URL
    ConfirmedSubscriber::parse("noreply@example.com".to_string())?,
    reqwest::Client::new(),
);

let email_client = web::Data::new(email_client);
```

### 실제 이메일 서비스 통합 예제

#### SendGrid 사용
```rust
// Cargo.toml
sendgrid = "0.16"

// src/email_client.rs 수정
pub async fn send_email(
    &self,
    recipient: &str,
    subject: &str,
    html_content: &str,
) -> Result<(), String> {
    let mail = Mail::new()
        .add_from(self.sender.inner().parse().unwrap())
        .add_to(recipient.parse().unwrap())
        .add_subject(subject)
        .add_html(html_content);

    let sender = Sender::new(env::var("SENDGRID_API_KEY").unwrap());
    sender.send(&mail).await
        .map_err(|e| format!("Failed to send email: {}", e))?;

    Ok(())
}
```

#### AWS SES 사용
```rust
// Cargo.toml
aws-sdk-ses = "1.0"

// src/email_client.rs 수정
pub async fn send_email(
    &self,
    recipient: &str,
    subject: &str,
    html_content: &str,
) -> Result<(), String> {
    let config = aws_config::load_from_env().await;
    let client = aws_sdk_ses::Client::new(&config);

    client
        .send_email()
        .source(self.sender.inner())
        .destination(
            Destination::builder()
                .to_addresses(recipient)
                .build()
        )
        .message(
            Message::builder()
                .subject(Content::builder().data(subject).build().unwrap())
                .body(Body::builder().html(html_content).build().unwrap())
                .build()
                .unwrap()
        )
        .send()
        .await
        .map_err(|e| format!("Failed to send email: {}", e))?;

    Ok(())
}
```

## 4. 애플리케이션 실행

### 개발 모드
```bash
cargo run
```

### 릴리스 모드
```bash
cargo build --release
./target/release/zero2prod
```

## 5. 가상 이메일 서버 구성 (테스트용)

### Mock Email Server 설치
```bash
# Postman Mock Server 또는 다음 간단한 로컬 서버 사용

# Node.js 기반 간단한 이메일 서버 예제
npm install express body-parser
```

### mock-email-server.js
```javascript
const express = require('express');
const bodyParser = require('body-parser');

const app = express();
app.use(bodyParser.json());

// 받은 이메일 저장 (메모리)
const emails = [];

app.post('/email', (req, res) => {
    const { to, Subject, Html } = req.body;

    console.log('\n=== EMAIL SENT ===');
    console.log(`To: ${to}`);
    console.log(`Subject: ${Subject}`);
    console.log(`Body: ${Html}`);
    console.log('==================\n');

    emails.push({ to, Subject, Html, timestamp: new Date() });

    res.status(200).json({ success: true, message: 'Email sent' });
});

app.get('/emails', (req, res) => {
    res.json(emails);
});

app.listen(3030, () => {
    console.log('Mock Email Server running on http://localhost:3030');
});
```

### 실행
```bash
node mock-email-server.js
```

## 6. API 테스트

### 1. 구독 요청
```bash
curl -X POST http://localhost:8000/subscriptions \
  -H "Content-Type: application/x-www-form-urlencoded" \
  -d "name=John Doe&email=john@example.com"
```

### 2. 확인 링크 생성 및 확인
```bash
# 데이터베이스에서 토큰 조회
psql -U postgres -d zero2prod -c "SELECT subscription_token FROM subscription_tokens LIMIT 1;"

# 확인 링크 클릭
curl "http://localhost:8000/subscriptions/confirm?token=<token>"
```

### 3. 구독 상태 확인
```bash
psql -U postgres -d zero2prod -c "SELECT id, email, status FROM subscriptions WHERE email='john@example.com';"
```

## 7. 로깅 및 모니터링

### 로그 레벨 설정
```bash
# Debug 모드
RUST_LOG=debug cargo run

# Info 모드
RUST_LOG=info cargo run

# Trace 모드
RUST_LOG=trace cargo run
```

### 로그 필터링
```bash
RUST_LOG=zero2prod=debug,actix_web=info cargo run
```

## 8. 문제 해결

### 마이그레이션 오류
```bash
# 마이그레이션 상태 확인
sqlx migrate info

# 특정 마이그레이션 재실행
sqlx migrate revert
sqlx migrate run
```

### 데이터베이스 연결 오류
```bash
# PostgreSQL 연결 테스트
psql postgresql://username:password@localhost:5432/zero2prod

# 환경 변수 확인
echo $DATABASE_URL
```

### 포트 사용 중 오류
```bash
# 사용 중인 포트 확인 및 종료
# Windows
netstat -ano | findstr :8000
taskkill /PID <PID> /F

# Linux/Mac
lsof -i :8000
kill -9 <PID>
```

## 9. 성능 최적화

### 인덱스 생성
```sql
-- 이미 마이그레이션에 포함됨
CREATE INDEX idx_subscriptions_email ON subscriptions(email);
CREATE INDEX idx_subscription_tokens_subscriber_id ON subscription_tokens(subscriber_id);
CREATE INDEX idx_subscriptions_status ON subscriptions(status);
```

### 연결 풀 설정
```rust
// src/startup.rs에서
let pool = PgPoolOptions::new()
    .max_connections(10)
    .connect(&database_url)
    .await
    .expect("Failed to create pool.");
```

## 10. 배포 체크리스트

- [ ] 환경 변수 설정
- [ ] 데이터베이스 마이그레이션 실행
- [ ] 이메일 서비스 설정 완료
- [ ] TLS/SSL 인증서 구성
- [ ] 로깅 시스템 설정
- [ ] 모니터링 도구 연결
- [ ] 백업 계획 수립
- [ ] 부하 테스트 완료
