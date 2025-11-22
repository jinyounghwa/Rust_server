# 빠른 시작 가이드 (Quick Start)

## 📋 이 문서의 목적

5분 안에 Zero2Prod 애플리케이션을 실행하고 테스트하세요.

---

## 🚀 5단계 실행

### 1단계: 환경 설정 (1분)

**필수 설치:**
- Rust 1.70+
- PostgreSQL 14+
- sqlx-cli

**설치:**
```bash
# Rust (이미 설치된 경우 생략)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# sqlx-cli
cargo install sqlx-cli --no-default-features --features postgres
```

### 2단계: 데이터베이스 준비 (2분)

```bash
# PostgreSQL 시작
# Windows: 서비스에서 시작 또는
psql -U postgres

# 데이터베이스 생성
CREATE DATABASE zero2prod;

# 연결 확인
\c zero2prod

# 마이그레이션 실행
sqlx migrate run
```

### 3단계: 애플리케이션 실행 (1분)

```bash
# 프로젝트 디렉토리로 이동
cd zero2prod

# 환경 변수 설정 (.env 파일)
DATABASE_URL=postgres://postgres:password@localhost:5432/zero2prod
RUST_LOG=info
SERVER_PORT=8000
SERVER_HOST=127.0.0.1

# 빌드 및 실행
cargo run
```

**출력:**
```
    Finished dev [unoptimized + debuginfo] target(s) in 1.2s
     Running `target/debug/zero2prod`
[INFO] Server running at http://127.0.0.1:8000
```

### 4단계: 테스트 (1분)

**1. 구독 생성:**
```bash
curl -X POST http://localhost:8000/subscriptions \
  -H "Content-Type: application/x-www-form-urlencoded" \
  -d "name=John Doe&email=john@example.com"
```

**응답 (성공):**
```json
HTTP 200 OK
```

**응답 (검증 실패 - 잘못된 이메일):**
```json
{
    "error_id": "uuid",
    "message": "email has invalid format",
    "code": "VALIDATION_ERROR",
    "status": 400,
    "timestamp": "2024-11-22T10:30:45.123Z"
}
```

**2. 건강 확인:**
```bash
curl http://localhost:8000/health_check
```

**응답:**
```json
HTTP 200 OK
```

**3. 이메일 확인 (선택사항):**
```bash
# 데이터베이스에서 토큰 확인
psql -U postgres -d zero2prod -c \
  "SELECT subscription_token FROM subscription_tokens LIMIT 1;"

# 확인 링크 클릭 (TOKEN을 실제 토큰으로 교체)
curl "http://localhost:8000/subscriptions/confirm?token=TOKEN"
```

### 5단계: 로그 확인 (자동)

**구조화된 로그 확인:**
```
[INFO] Processing new subscription
[INFO] New subscriber saved successfully
[INFO] Confirmation email sent successfully
[INFO] Subscription created successfully
```

**검증 실패 로그:**
```
[WARN] Validation error: email has invalid format
```

**데이터베이스 오류 로그:**
```
[ERROR] Failed request
  request_id: uuid
  error_type: DatabaseError
  response_status: 409 (중복)
```

---

## 🧪 테스트 데이터

### 유효한 이메일
```
user@example.com
test.email@domain.co.uk
user+tag@example.com
```

### 유효한 이름
```
John Doe
Jean-Pierre
O'Brien
```

### 무효한 입력 (테스트)
```
# 빈 이메일
curl -X POST http://localhost:8000/subscriptions \
  -d "name=John&email="

# 잘못된 이메일
curl -X POST http://localhost:8000/subscriptions \
  -d "name=John&email=invalid"

# SQL 주입 시도
curl -X POST http://localhost:8000/subscriptions \
  -d "name='; DROP TABLE--&email=test@example.com"

# 매우 긴 입력 (DoS)
curl -X POST http://localhost:8000/subscriptions \
  -d "name=$(python3 -c 'print(\"a\" * 1000)')&email=test@example.com"
```

---

## 🔍 로그 확인

### 실시간 로그 확인 (터미널)
```bash
# 애플리케이션 실행 중 로그 확인
RUST_LOG=debug cargo run
```

### 로그 파일에서 찾기
```bash
# 오류 찾기
grep ERROR /var/log/application.log

# request_id로 추적
grep "request_id=uuid" /var/log/application.log

# 특정 엔드포인트 오류
grep "/subscriptions" /var/log/application.log | grep ERROR
```

---

## 📊 모니터링

### 1. Curl로 직접 모니터링

**실시간 요청:**
```bash
# 터미널 1: 애플리케이션 실행
cargo run

# 터미널 2: 요청 보내기
watch -n 1 'curl -X POST http://localhost:8000/subscriptions \
  -d "name=Test&email=test$(date +%s)@example.com"'
```

### 2. 통계 수집

**성공/실패 비율:**
```bash
# 10개의 구독 시도
for i in {1..10}; do
  curl -X POST http://localhost:8000/subscriptions \
    -d "name=User$i&email=user$i@example.com" \
    -s -o /dev/null -w "%{http_code}\n"
done
```

---

## ⚠️ 트러블슈팅

### 문제: "Connection refused"
```bash
# 해결: PostgreSQL이 실행 중인지 확인
psql -U postgres -c "SELECT 1;"

# 또는 PostgreSQL 시작
sudo systemctl start postgresql  # Linux
brew services start postgresql  # macOS
# Windows: Services에서 시작
```

### 문제: "Database does not exist"
```bash
# 해결: 데이터베이스 생성
psql -U postgres -c "CREATE DATABASE zero2prod;"
```

### 문제: "Migration failed"
```bash
# 해결: 마이그레이션 다시 실행
sqlx migrate run --database-url postgres://postgres:password@localhost:5432/zero2prod
```

### 문제: "Port 8000 already in use"
```bash
# 해결: 다른 포트 사용
SERVER_PORT=8001 cargo run

# 또는 기존 프로세스 종료
kill -9 $(lsof -t -i:8000)
```

### 문제: 로그가 안 보임
```bash
# 해결: RUST_LOG 설정
RUST_LOG=info cargo run
RUST_LOG=debug cargo run  # 더 상세한 로그
```

---

## 🎯 주요 엔드포인트

| 메서드 | 경로 | 설명 | 기대 응답 |
|--------|------|------|----------|
| GET | `/health_check` | 상태 확인 | 200 OK |
| POST | `/subscriptions` | 구독 생성 | 200 OK 또는 400/409/503 |
| GET | `/subscriptions/confirm?token=TOKEN` | 확인 | 200 OK 또는 400 |

---

## 📈 다음 단계

### 기능 탐색
1. ✅ 건강 확인 엔드포인트 - 완료
2. ✅ 구독 생성 - 완료
3. ✅ 이메일 확인 - 완료

### 로깅 확인
1. 검증 실패 로그 확인
2. 데이터베이스 오류 로그 확인
3. 요청 ID로 전체 흐름 추적

### 보안 테스트
1. SQL 주입 시도
2. 매우 긴 입력 (DoS)
3. 제어 문자 검사

### 성능 테스트
```bash
# 부하 테스트 (apache bench)
ab -n 1000 -c 10 http://localhost:8000/health_check

# 또는
cargo install flamegraph
cargo flamegraph
```

---

## 📚 상세 문서

더 깊이 있는 내용은 다음 문서를 참조하세요:

| 문서 | 내용 |
|------|------|
| `docs/ERROR_HANDLING.md` | 오류 처리 완전 가이드 |
| `docs/REQUEST_FAILURE_LOGGING.md` | 로깅 시스템 완전 가이드 |
| `docs/SECURITY.md` | 보안 기능 상세 설명 |
| `docs/EMAIL_CONFIRMATION_SERVICE.md` | 이메일 확인 서비스 |

---

## ✨ 팁과 트릭

### 환경 변수 파일 사용
```bash
# .env 파일 생성
cat > .env << EOF
DATABASE_URL=postgres://postgres:password@localhost:5432/zero2prod
RUST_LOG=info
SERVER_PORT=8000
SERVER_HOST=127.0.0.1
EOF

# 로드
source .env  # Linux/macOS
# Windows: 환경 변수에 직접 설정
```

### 자동 리로드
```bash
cargo install cargo-watch
cargo watch -x run
```

### 테스트 데이터 생성
```bash
# 100개의 구독 생성
for i in {1..100}; do
  curl -X POST http://localhost:8000/subscriptions \
    -d "name=User$i&email=user$i@example.com" \
    -s -o /dev/null
done

# 데이터베이스에서 확인
psql -U postgres -d zero2prod -c "SELECT COUNT(*) FROM subscriptions;"
```

### 로그를 파일에 저장
```bash
RUST_LOG=info cargo run 2>&1 | tee application.log
```

---

## 🎉 완료!

이제 Zero2Prod 애플리케이션이 실행 중입니다! 🚀

- ✅ 애플리케이션 실행
- ✅ 구독 생성
- ✅ 이메일 확인
- ✅ 로그 확인
- ✅ 오류 처리 테스트

---

**마지막 업데이트:** 2024-11-22
**필요한 시간:** 약 5분
**난이도:** ⭐⭐ (초급)
