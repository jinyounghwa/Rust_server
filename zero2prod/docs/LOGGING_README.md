# 로깅 문서 모음

이 디렉토리에는 zero2prod 프로젝트의 구조화된 로깅 시스템에 대한 완전한 문서가 포함되어 있습니다.

## 📚 문서 목록

### 1. **[LOGGING_QUICKSTART.md](./LOGGING_QUICKSTART.md)** ⭐ 새로운 사용자용
- **소요 시간:** 5분
- **내용:**
  - 프로젝트 실행 방법
  - 기본적인 로그 레벨 설정
  - 자주 사용하는 명령어
  - 실제 로그 출력 예제
- **추천:** 처음 시작하는 개발자

```bash
# 빠른 시작
RUST_LOG=info cargo run
```

---

### 2. **[STRUCTURED_LOGGING.md](./STRUCTURED_LOGGING.md)** 📖 완전한 가이드
- **소요 시간:** 30분
- **내용:**
  - 구조화된 로깅의 개념 및 이점
  - 아키텍처 설명
  - 로그 작성 방법 (기본 및 고급)
  - 로그 필드 포맷터 설명
  - 실제 코드 예제
  - 로그 레벨 제어
  - 모범 사례 (베스트 프랙티스)
  - 트러블슈팅 가이드
- **추천:** 로깅 시스템을 깊이 있게 이해하고 싶은 개발자

### 주요 섹션

#### 📝 로그 작성 예제
```rust
// 기본 로깅
tracing::info!("Application started");

// 구조화된 필드 포함
tracing::info!(
    subscriber_id = %subscriber_id,
    email = %email,
    "New subscriber saved successfully"
);
```

#### 🎯 로그 레벨
- **TRACE:** 가장 상세 (개발용)
- **DEBUG:** 디버깅 정보
- **INFO:** 일반 정보 (기본값)
- **WARN:** 경고
- **ERROR:** 에러

---

### 3. **[LOGGING_IMPLEMENTATION.md](./LOGGING_IMPLEMENTATION.md)** 🔧 기술 세부사항
- **소요 시간:** 20분
- **내용:**
  - 구현 파일 구조
  - 각 파일별 상세 설명
  - 초기화 흐름 (플로우 다이어그램 포함)
  - 모듈별 로깅 구현 방식
  - 통합 테스트 방법
  - 성능 고려사항
  - 향후 개선 사항
- **추천:** 코드 구조를 이해하고 싶거나 새로운 모듈을 추가하는 개발자

### 주요 파일 설명

| 파일 | 역할 |
|------|------|
| `src/telemetry.rs` | 로깅 시스템 초기화 |
| `src/logger.rs` | HTTP 요청/응답 로깅 미들웨어 |
| `src/main.rs` | 애플리케이션 생명주기 로깅 |
| `src/routes/subscriptions.rs` | 비즈니스 로직 로깅 |
| `src/routes/health_check.rs` | 헬스 체크 로깅 |

---

## 🚀 빠른 시작

### 설치 및 실행

```bash
# 1. 기본 설정으로 실행
RUST_LOG=info cargo run

# 2. 디버그 모드로 실행 (더 상세한 로그)
RUST_LOG=debug cargo run

# 3. 테스트 중 로그 보기
RUST_LOG=info cargo test -- --nocapture
```

### 로그 레벨 설정

```bash
# INFO (권장 - 기본값)
RUST_LOG=info cargo run

# DEBUG (개발 중 디버깅용)
RUST_LOG=debug cargo run

# 특정 모듈만 DEBUG
RUST_LOG=zero2prod=debug,info cargo run

# ERROR (프로덕션 - 최소 로그)
RUST_LOG=error cargo run --release
```

---

## 📊 로그 출력 예제

### 성공한 구독 요청
```json
{"timestamp":"2025-11-14T10:00:05.235Z","level":"INFO","message":"Processing new subscription","email":"john@example.com","name":"John Doe"}
{"timestamp":"2025-11-14T10:00:05.267Z","level":"INFO","message":"New subscriber saved successfully","subscriber_id":"550e8400-e29b-41d4-a716-446655440000","email":"john@example.com"}
```

### 실패한 요청 (검증 오류)
```json
{"timestamp":"2025-11-14T10:00:10.346Z","level":"WARN","message":"Invalid subscription request received","name_valid":true,"email_valid":false}
```

---

## 📋 로그 필드 참고

### HTTP 로깅 필드
| 필드 | 설명 | 예 |
|------|------|-----|
| `method` | HTTP 메서드 | POST |
| `path` | 요청 경로 | /subscriptions |
| `status` | HTTP 상태 코드 | 200 |
| `elapsed_ms` | 응답 시간 | 45 |

### 구독 로깅 필드
| 필드 | 설명 | 예 |
|------|------|-----|
| `subscriber_id` | 구독자 ID | 550e8400-... |
| `email` | 이메일 주소 | user@example.com |
| `name` | 사용자 이름 | John Doe |
| `error` | 에러 메시지 | duplicate key value... |

---

## 🛠️ 일반적인 작업

### 로그를 파일로 저장
```bash
RUST_LOG=info cargo run > app.log 2>&1
```

### 로그 필터링 (jq 사용)
```bash
# ERROR 로그만 보기
RUST_LOG=info cargo run 2>&1 | jq 'select(.level == "ERROR")'

# 느린 요청 (100ms 이상) 보기
RUST_LOG=info cargo run 2>&1 | jq 'select(.elapsed_ms > 100)'

# 특정 이메일의 모든 로그
RUST_LOG=info cargo run 2>&1 | jq 'select(.email == "user@example.com")'
```

### 로그 통계
```bash
# 로그 레벨별 개수
RUST_LOG=info cargo run 2>&1 | jq -s 'group_by(.level) | map({level: .[0].level, count: length})'

# 가장 자주 나오는 메시지
RUST_LOG=info cargo run 2>&1 | jq -s 'group_by(.message) | map({message: .[0].message, count: length}) | sort_by(.count) | reverse'
```

---

## ❓ FAQ

### Q: 기본 로그 레벨은?
**A:** `info` - `RUST_LOG` 환경 변수가 설정되지 않으면 자동으로 `info` 레벨이 적용됩니다.

### Q: 프로덕션 환경에서 추천되는 설정은?
**A:** `RUST_LOG=warn` - 경고와 에러만 기록하여 최소 오버헤드로 운영합니다.

### Q: 로그가 JSON 형식이 아니라면?
**A:** `src/telemetry.rs`의 `init_telemetry()` 함수가 `main()` 맨 처음에 호출되었는지 확인하세요.

### Q: 특정 모듈의 로그만 보고 싶어요
**A:** `RUST_LOG=module_name=level` 형식으로 설정합니다.
```bash
RUST_LOG=zero2prod::routes=debug,info cargo run
```

### Q: 테스트 중 로그가 보이지 않아요
**A:** `--nocapture` 플래그를 추가하세요.
```bash
RUST_LOG=info cargo test -- --nocapture
```

---

## 📈 로깅 아키텍처

```
┌─────────────────────────────────────────────┐
│          Application Code                    │
│  tracing::info!(), tracing::error!() 등      │
└────────────────────┬────────────────────────┘
                     │
┌────────────────────▼────────────────────────┐
│    Tracing Subscriber (텔레메트리 시스템)    │
│ ┌──────────────┐    ┌────────────────────┐ │
│ │ EnvFilter    │    │ JSON Formatter     │ │
│ │ (RUST_LOG)   │    │ (.json())          │ │
│ └──────────────┘    └────────────────────┘ │
└────────────────────┬────────────────────────┘
                     │
             JSON 형식의 로그
                     ↓
        ┌────────────────────┐
        │ stdout (화면)      │
        │ 파일               │
        │ 로그 집계 시스템   │
        │ (ELK, Datadog)     │
        └────────────────────┘
```

---

## 🔄 생명주기 로그 흐름

```
애플리케이션 시작
    ↓
[INFO] Starting application
    ↓
[INFO] Configuration loaded successfully
    ↓
[INFO] Attempting to connect to database
    ↓
[INFO] Database connection pool created successfully
    ↓
[INFO] Server started successfully
    ↓
─────────────────────────────────────────────
[INFO] HTTP request received
[INFO] Processing new subscription
[INFO] New subscriber saved successfully
[INFO] HTTP request completed
─────────────────────────────────────────────
    ↓
서버 정지
```

---

## 📚 추가 리소스

### 공식 문서
- [Tracing Documentation](https://docs.rs/tracing/)
- [Tracing Subscriber Documentation](https://docs.rs/tracing-subscriber/)

### 실습
1. [LOGGING_QUICKSTART.md](./LOGGING_QUICKSTART.md)에서 기본 사용법 학습
2. [STRUCTURED_LOGGING.md](./STRUCTURED_LOGGING.md)에서 고급 기능 학습
3. [LOGGING_IMPLEMENTATION.md](./LOGGING_IMPLEMENTATION.md)에서 기술 세부사항 학습

---

## 📊 문서 선택 가이드

```
시작하기
    │
    ├─ "5분 안에 시작하고 싶어요"
    │  └─ LOGGING_QUICKSTART.md ⭐
    │
    ├─ "로깅을 제대로 이해하고 싶어요"
    │  └─ STRUCTURED_LOGGING.md 📖
    │
    └─ "코드 구조를 알고 싶어요"
       └─ LOGGING_IMPLEMENTATION.md 🔧
```

---

## 💡 팁

### 개발 중 추천 설정
```bash
# 디버깅할 때 가장 유용한 조합
RUST_LOG=zero2prod=debug,info cargo run
```

### 프로덕션 배포
```bash
# 최소 오버헤드로 운영
RUST_LOG=warn cargo run --release
```

### 로그 저장
```bash
# 현재 실행의 모든 로그를 파일에 저장
RUST_LOG=info cargo run 2>&1 | tee app-$(date +%s).log
```

---

## 🔧 트러블슈팅

| 문제 | 해결책 |
|------|--------|
| 로그가 보이지 않음 | `RUST_LOG=info cargo run` 확인 |
| 로그가 너무 많음 | `RUST_LOG=warn cargo run` 설정 |
| JSON 형식이 아님 | `init_telemetry()` 호출 확인 |
| 테스트 중 로그 미표시 | `--nocapture` 플래그 추가 |

---

## 📝 버전 정보

| 문서 | 버전 | 수정일 |
|------|------|--------|
| LOGGING_QUICKSTART.md | 1.0 | 2025-11-14 |
| STRUCTURED_LOGGING.md | 1.0 | 2025-11-14 |
| LOGGING_IMPLEMENTATION.md | 1.0 | 2025-11-14 |

---

**마지막 업데이트:** 2025-11-14

💬 **질문이나 제안이 있으신가요?** GitHub Issues에 남겨주세요!
