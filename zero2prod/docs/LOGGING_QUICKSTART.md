# 구조화된 로깅 빠른 시작 가이드

## 5분 안에 시작하기

### 1단계: 프로젝트 실행 (기본 설정)

```bash
# INFO 레벨로 구조화된 로그 출력
cargo run
```

**출력 예:**
```json
{"timestamp":"2025-11-14T10:00:00.000Z","level":"INFO","message":"Starting application"}
{"timestamp":"2025-11-14T10:00:00.001Z","level":"INFO","message":"Configuration loaded successfully"}
{"timestamp":"2025-11-14T10:00:00.154Z","level":"INFO","message":"Server started successfully"}
```

### 2단계: 다른 로그 레벨 시도

```bash
# DEBUG 레벨 (더 상세함)
RUST_LOG=debug cargo run

# ERROR 레벨만 (가장 적음)
RUST_LOG=error cargo run

# TRACE 레벨 (매우 상세 - 개발 중에만 사용)
RUST_LOG=trace cargo run
```

### 3단계: 특정 모듈의 로그만 보기

```bash
# zero2prod 모듈만 DEBUG, 나머지는 WARN
RUST_LOG=zero2prod=debug,warn cargo run

# subscriptions 모듈만 TRACE
RUST_LOG=zero2prod::routes::subscriptions=trace,info cargo run
```

### 4단계: 테스트 중 로그 보기

```bash
# 테스트 실행 중 로그 출력
RUST_LOG=info cargo test -- --nocapture

# 특정 테스트의 로그만 보기
RUST_LOG=debug cargo test health_check_works -- --nocapture
```

---

## 로그 레벨 참고표

```
┌─────────┬──────────────────────────────────┐
│ 레벨    │ 언제 사용?                       │
├─────────┼──────────────────────────────────┤
│ TRACE   │ 모든 프로그래밍 흐름을 추적할 때 │
│         │ (매우 느리고 상세함)             │
├─────────┼──────────────────────────────────┤
│ DEBUG   │ 개발 중 버그 찾을 때             │
├─────────┼──────────────────────────────────┤
│ INFO ✓  │ 중요한 이벤트를 알고 싶을 때    │
│ (기본)  │ (프로덕션 권장)                  │
├─────────┼──────────────────────────────────┤
│ WARN    │ 뭔가 이상한 일이 발생했을 때     │
│         │ (하지만 에러는 아닐 때)          │
├─────────┼──────────────────────────────────┤
│ ERROR   │ 문제가 발생했을 때               │
│         │ (가장 심각함)                    │
└─────────┴──────────────────────────────────┘
```

---

## 자주 쓰는 명령어

### 로컬 개발 중

```bash
# 추천: 현재 설정 (가장 적절한 양의 로그)
RUST_LOG=info cargo run

# 버그 찾을 때
RUST_LOG=debug cargo run

# 특정 기능 집중 분석
RUST_LOG=zero2prod::routes=debug,info cargo run
```

### 테스트 중

```bash
# 모든 테스트 실행 + 로그 출력
RUST_LOG=info cargo test -- --nocapture

# 특정 테스트만 + 상세 로그
RUST_LOG=debug cargo test subscribe_returns_a_200 -- --nocapture
```

### 프로덕션 배포 시

```bash
# 경고와 에러만 로그 (최소 오버헤드)
RUST_LOG=warn cargo run --release

# 또는 릴리스 빌드 후
RUST_LOG=warn ./target/release/zero2prod
```

---

## 로그 필터링 예제

### jq를 사용한 실시간 분석

```bash
# 1. 모든 로그 보기
RUST_LOG=info cargo run 2>&1 | jq .

# 2. ERROR 로그만 보기
RUST_LOG=info cargo run 2>&1 | jq 'select(.level == "ERROR")'

# 3. POST 요청만 보기
RUST_LOG=info cargo run 2>&1 | jq 'select(.method == "POST")'

# 4. 느린 요청 (100ms 이상) 보기
RUST_LOG=info cargo run 2>&1 | jq 'select(.elapsed_ms > 100)'

# 5. 특정 이메일 관련 모든 로그
RUST_LOG=info cargo run 2>&1 | jq 'select(.email == "user@example.com")'

# 6. 특정 필드만 추출
RUST_LOG=info cargo run 2>&1 | jq '{timestamp, level, message, path: .path}'
```

---

## 실제 시나리오별 명령어

### 시나리오 1: 애플리케이션이 느림

```bash
# 각 요청이 얼마나 걸리는지 확인
RUST_LOG=info cargo run 2>&1 | jq 'select(.level == "INFO") | {path, elapsed_ms, status}'
```

### 시나리오 2: 특정 이메일의 문제 추적

```bash
# 특정 사용자의 모든 활동 로그
TARGET_EMAIL="user@example.com"
RUST_LOG=info cargo run 2>&1 | jq --arg email "$TARGET_EMAIL" 'select(.email == $email)'
```

### 시나리오 3: 데이터베이스 에러 디버깅

```bash
# 모든 에러 메시지 보기
RUST_LOG=info cargo run 2>&1 | jq 'select(.level == "ERROR") | {message, error, subscriber_id}'
```

### 시나리오 4: API 응답 시간 분석

```bash
# 각 엔드포인트별 평균 응답 시간
RUST_LOG=info cargo run 2>&1 | jq -s 'group_by(.path) | map({
  path: .[0].path,
  avg_ms: (map(.elapsed_ms) | add / length),
  count: length
})'
```

---

## 로그 출력 예제

### 정상적인 구독 요청

```bash
$ RUST_LOG=info cargo run &
$ curl -X POST http://127.0.0.1:8000/subscriptions \
  -d "name=John+Doe&email=john@example.com"
```

**로그 출력:**
```json
{"timestamp":"2025-11-14T10:00:05.234Z","level":"INFO","message":"HTTP request received","method":"POST","path":"/subscriptions"}
{"timestamp":"2025-11-14T10:00:05.235Z","level":"INFO","message":"Processing new subscription","email":"john@example.com","name":"John Doe"}
{"timestamp":"2025-11-14T10:00:05.267Z","level":"INFO","message":"New subscriber saved successfully","subscriber_id":"550e8400-e29b-41d4-a716-446655440000","email":"john@example.com"}
{"timestamp":"2025-11-14T10:00:05.268Z","level":"INFO","message":"HTTP request completed","method":"POST","path":"/subscriptions","status":200,"elapsed_ms":34}
```

### 잘못된 요청 (이메일 누락)

```bash
$ curl -X POST http://127.0.0.1:8000/subscriptions \
  -d "name=John+Doe"
```

**로그 출력:**
```json
{"timestamp":"2025-11-14T10:00:10.345Z","level":"INFO","message":"HTTP request received","method":"POST","path":"/subscriptions"}
{"timestamp":"2025-11-14T10:00:10.346Z","level":"WARN","message":"Invalid subscription request received","name_valid":true,"email_valid":false}
{"timestamp":"2025-11-14T10:00:10.346Z","level":"INFO","message":"HTTP request completed","method":"POST","path":"/subscriptions","status":400,"elapsed_ms":1}
```

### 헬스 체크

```bash
$ RUST_LOG=debug cargo run &
$ curl http://127.0.0.1:8000/health_check
```

**로그 출력:**
```json
{"timestamp":"2025-11-14T10:00:15.123Z","level":"INFO","message":"HTTP request received","method":"GET","path":"/health_check"}
{"timestamp":"2025-11-14T10:00:15.124Z","level":"DEBUG","message":"Health check endpoint called"}
{"timestamp":"2025-11-14T10:00:15.124Z","level":"INFO","message":"HTTP request completed","method":"GET","path":"/health_check","status":200,"elapsed_ms":1}
```

---

## 팁과 트릭

### 팁 1: 로그를 파일에 저장하기

```bash
# 방법 1: 쉘 리다이렉션
RUST_LOG=info cargo run > app.log 2>&1

# 방법 2: tee 사용 (화면과 파일 모두에 출력)
RUST_LOG=info cargo run 2>&1 | tee app.log
```

### 팁 2: 특정 시간의 로그만 보기

```bash
# 지난 5분의 로그만 보기
RUST_LOG=info cargo run 2>&1 | jq "select(.timestamp > \"$(date -u -d '-5 minutes' +%Y-%m-%dT%H:%M:%S.000Z)\")"
```

### 팁 3: 로그 통계 보기

```bash
# 로그 레벨별 개수
RUST_LOG=info cargo run 2>&1 | jq -s 'group_by(.level) | map({level: .[0].level, count: length})'

# 가장 자주 나오는 메시지
RUST_LOG=info cargo run 2>&1 | jq -s 'group_by(.message) | map({message: .[0].message, count: length}) | sort_by(.count) | reverse'
```

### 팁 4: 컬러 출력 (jq 사용)

```bash
# 컬러로 보기 (터미널이 지원할 때)
RUST_LOG=info cargo run 2>&1 | jq -C .

# 더 예쁘게 보기 (선택 포맷)
RUST_LOG=info cargo run 2>&1 | jq '[.timestamp, .level, .message] | @csv'
```

---

## 환경 변수 빠른 참조

```bash
# 기본 (추천)
RUST_LOG=info cargo run

# 개발
RUST_LOG=debug cargo run

# 분석
RUST_LOG=trace cargo run

# 프로덕션
RUST_LOG=warn cargo run --release

# 복합 설정
RUST_LOG=zero2prod=debug,sqlx=info,warn cargo run
```

---

## 일반적인 실수와 해결책

### 실수 1: 로그가 JSON이 아님

```bash
# ❌ 문제: 텍스트 로그로 출력됨
cargo run

# ✅ 해결: 반드시 RUST_LOG를 설정
RUST_LOG=info cargo run
```

### 실수 2: 로그가 너무 많음

```bash
# ❌ 문제: 화면이 로그로 가득 참
RUST_LOG=trace cargo run

# ✅ 해결: 로그 레벨 상향
RUST_LOG=warn cargo run
```

### 실수 3: 특정 로그를 못 찾음

```bash
# ❌ 문제: 로그가 필터링되어 보이지 않음
RUST_LOG=error cargo run

# ✅ 해결: 필요한 레벨까지 낮춤
RUST_LOG=info cargo run
```

### 실수 4: 테스트 중 로그가 안 보임

```bash
# ❌ 문제: 기본 테스트 실행
cargo test

# ✅ 해결: nocapture 플래그 추가
RUST_LOG=info cargo test -- --nocapture
```

---

## 다음 단계

1. **[구조화된 로깅 완전 가이드](./STRUCTURED_LOGGING.md)** - 자세한 설명 및 모범 사례
2. **[구현 세부사항](./LOGGING_IMPLEMENTATION.md)** - 코드 구조 및 아키텍처

---

## 빠른 참고표

| 작업 | 명령어 |
|------|--------|
| 기본 실행 | `RUST_LOG=info cargo run` |
| 디버깅 | `RUST_LOG=debug cargo run` |
| 테스트 | `RUST_LOG=info cargo test -- --nocapture` |
| 프로덕션 | `RUST_LOG=warn cargo run --release` |
| ERROR만 | `RUST_LOG=error cargo run` |
| 예쁘게 보기 | `RUST_LOG=info cargo run 2>&1 \| jq .` |
| 파일 저장 | `RUST_LOG=info cargo run > app.log 2>&1` |

---

**마지막 수정:** 2025-11-14
**버전:** 1.0
