# 아키텍처 및 데이터 흐름 (Architecture and Data Flow)

## 시스템 아키텍처

### 전체 레이어 구조

```
┌─────────────────────────────────────────────────┐
│              Client Application                  │
│         (Browser, Mobile, API Client)            │
└───────────────────┬─────────────────────────────┘
                    │
                    │ HTTP Request
                    ↓
        ┌──────────────────────┐
        │   Load Balancer      │ (선택사항)
        └──────────┬───────────┘
                   │
        ┌──────────────────────┐
        │   Reverse Proxy      │ (nginx, Cloudflare)
        │  • HTTPS Termination │
        │  • Compression       │
        │  • Caching           │
        └──────────┬───────────┘
                   │
        ┌──────────────────────────────────────┐
        │    Actix-web Framework               │
        │  ┌────────────────────────────────┐  │
        │  │ 1. LoggerMiddleware            │  │ (요청 로깅)
        │  │ 2. SecurityMiddleware (향후)    │  │ (보안 헤더)
        │  │ 3. Route Handler               │  │
        │  └────────────────────────────────┘  │
        └──────────┬───────────────────────────┘
                   │
        ┌──────────────────────────────────────┐
        │    Request Handler: subscribe()       │
        │  ┌────────────────────────────────┐  │
        │  │ 1. FormData 파싱               │  │
        │  │ 2. 검증 모듈 호출              │  │
        │  │ 3. DB 삽입                     │  │
        │  │ 4. 응답 반환                  │  │
        │  └────────────────────────────────┘  │
        └──────────┬───────────────────────────┘
                   │
        ┌──────────────────────────────────────┐
        │    Validators Module                 │
        │  ┌────────────────────────────────┐  │
        │  │ • is_valid_email()             │  │
        │  │ • is_valid_name()              │  │
        │  │ • SQL 패턴 감지                │  │
        │  │ • 피싱 패턴 감지               │  │
        │  └────────────────────────────────┘  │
        └──────────┬───────────────────────────┘
                   │
        ┌──────────────────────────────────────┐
        │    SQLx (Type-safe SQL)              │
        │  ┌────────────────────────────────┐  │
        │  │ • 매개변수 바인딩              │  │
        │  │ • 컴파일타임 검증             │  │
        │  │ • SQL 인젝션 방지             │  │
        │  └────────────────────────────────┘  │
        └──────────┬───────────────────────────┘
                   │
        ┌──────────────────────────────────────┐
        │    PostgreSQL Database               │
        │  ┌────────────────────────────────┐  │
        │  │ subscriptions table             │  │
        │  │  • id (UUID)                   │  │
        │  │  • email (TEXT, UNIQUE)        │  │
        │  │  • name (TEXT)                 │  │
        │  │  • subscribed_at (TIMESTAMPTZ) │  │
        │  └────────────────────────────────┘  │
        └──────────────────────────────────────┘
```

---

## 요청 처리 흐름

### 단계별 프로세스

```
1. 클라이언트 요청
   ↓
   POST /subscriptions
   Content-Type: application/x-www-form-urlencoded
   name=John&email=john@example.com

2. Actix-web 프레임워크
   ↓
   • LoggerMiddleware: 요청 로깅
   • Route 매칭: subscribe() 핸들러 호출

3. HTTP 리퀘스트 파싱
   ↓
   FormData 추출:
   FormData {
       name: Some("John"),
       email: Some("john@example.com")
   }

4. 입력 검증 (validators.rs)
   ↓
   4.1 이메일 검증
       • 길이: 5-254자 ✓
       • 형식: RFC 5322 ✓
       • 패턴: SQL 인젝션 없음 ✓
       • 피싱: 안전함 ✓
       → "john@example.com" 통과

   4.2 이름 검증
       • 길이: 1-256자 ✓
       • 패턴: 제어문자 없음 ✓
       • SQL: 위험 없음 ✓
       → "John" 통과

5. 데이터베이스 작업
   ↓
   5.1 UUID 생성
       uuid::Uuid::new_v4()
       → 550e8400-e29b-41d4-a716-446655440000

   5.2 현재 시간
       chrono::Utc::now()
       → 2025-11-20T10:30:00Z

   5.3 매개변수화 쿼리 실행
       INSERT INTO subscriptions
       VALUES ($1, $2, $3, $4)
       WHERE:
       $1 = UUID
       $2 = "john@example.com"
       $3 = "John"
       $4 = timestamp

6. 응답 반환
   ↓
   HTTP/1.1 200 OK
   Content-Length: 0

7. 로깅
   ↓
   {
     "timestamp": "2025-11-20T10:30:00Z",
     "level": "INFO",
     "message": "New subscriber saved successfully",
     "subscriber_id": "550e8400-e29b-41d4-a716-446655440000"
   }
```

---

## 보안 검증 흐름

### 다층 검증 체계

```
입력 데이터
    ↓
┌─────────────────────┐
│ 1. 길이 검증        │
│ (DoS 방어)          │
└──────┬──────────────┘
       │
       ├─→ 너무 짧음? → 400
       ├─→ 너무 김? → 400
       │
       ↓
┌─────────────────────┐
│ 2. 형식 검증        │
│ (RFC 5322)          │
└──────┬──────────────┘
       │
       ├─→ 형식 오류? → 400
       │
       ↓
┌─────────────────────┐
│ 3. 패턴 감지        │
│ (SQL 인젝션)        │
└──────┬──────────────┘
       │
       ├─→ UNION? → 400
       ├─→ --; /*? → 400
       ├─→ 함수? → 400
       │
       ↓
┌─────────────────────┐
│ 4. 피싱 감지        │
│ (이메일)            │
└──────┬──────────────┘
       │
       ├─→ 긴 로컬 파트? → 400
       ├─→ 다중 @? → 400
       ├─→ Null 바이트? → 400
       │
       ↓
┌─────────────────────┐
│ 5. 기타 검사        │
│ (이름)              │
└──────┬──────────────┘
       │
       ├─→ 제어문자? → 400
       ├─→ 특수문자 과다? → 400
       │
       ↓
┌──────────────────────┐
│ ✓ 유효한 데이터     │
│ → DB 삽입           │
└─────────────────────┘
```

---

## 모듈 간 상호작용

### 모듈 의존성 그래프

```
main.rs
    │
    ├─→ startup.rs
    │   ├─→ routes/subscriptions.rs
    │   │   ├─→ validators.rs
    │   │   │   ├─→ regex crate
    │   │   │   └─→ lazy_static crate
    │   │   │
    │   │   ├─→ sqlx crate
    │   │   │   └─→ PostgreSQL
    │   │   │
    │   │   └─→ security.rs
    │   │       └─→ std::time
    │   │
    │   └─→ logger.rs
    │
    ├─→ configuration.rs
    │   └─→ config crate
    │
    └─→ telemetry.rs
        └─→ tracing crate
```

---

## 에러 처리 흐름

### 에러 응답 매핑

```
입력 오류
    │
    ├─→ ValidationError::EmptyField
    │   → 400 Bad Request
    │
    ├─→ ValidationError::TooLong
    │   → 400 Bad Request
    │
    ├─→ ValidationError::InvalidFormat
    │   → 400 Bad Request
    │
    ├─→ ValidationError::PossibleSQLInjection
    │   → 400 Bad Request
    │
    └─→ ValidationError::SuspiciousContent
        → 400 Bad Request

데이터베이스 오류
    │
    ├─→ UNIQUE 제약 위반 (중복 이메일)
    │   → 409 Conflict
    │
    ├─→ 연결 오류
    │   → 500 Internal Server Error
    │
    └─→ 기타 DB 오류
        → 500 Internal Server Error
```

---

## 데이터 흐름 예시

### 성공 케이스: john@example.com 구독

```
클라이언트 요청:
┌──────────────────────────────────┐
│ POST /subscriptions              │
│ Content-Type: ...urlencoded      │
│                                  │
│ name=John&email=john@example.com │
└──────────────────────────────────┘

↓ (HTTP 전송)

Actix-web 수신:
┌──────────────────────────────────┐
│ FormData {                       │
│   name: Some("John"),            │
│   email: Some("john@example.com")│
│ }                                │
└──────────────────────────────────┘

↓ (검증)

Validators::is_valid_email()
├─ trim: "john@example.com"
├─ 길이: 19자 ✓
├─ 형식: RFC 5322 ✓
├─ SQL 패턴: 없음 ✓
└─ 피싱: 안전 ✓
Result: Ok("john@example.com")

Validators::is_valid_name()
├─ trim: "John"
├─ 길이: 4자 ✓
├─ 패턴: 안전 ✓
└─ SQL: 없음 ✓
Result: Ok("John")

↓ (DB 삽입)

SQLx::query
├─ UUID: 550e8400-e29b-41d4-...
├─ Email: "john@example.com"
├─ Name: "John"
└─ Time: 2025-11-20T10:30:00Z

PostgreSQL INSERT
├─ 검증: UNIQUE email ✓
├─ 저장: OK
└─ Result: 1 row inserted

↓ (응답)

┌──────────────────────────────┐
│ HTTP/1.1 200 OK              │
│ Content-Length: 0            │
│ X-Content-Type-Options: ...  │ (보안 헤더)
│ ...                          │
│                              │
│ (empty body)                 │
└──────────────────────────────┘

↓ (로깅)

{
  "timestamp": "2025-11-20T10:30:00Z",
  "level": "INFO",
  "message": "New subscriber saved successfully",
  "subscriber_id": "550e8400-e29b-41d4-a716-446655440000"
}
```

### 실패 케이스 1: SQL 인젝션

```
클라이언트 요청:
email=admin' UNION SELECT * FROM--@example.com

↓

Validators::is_valid_email()
├─ trim: "admin' UNION SELECT * FROM--@example.com"
├─ 길이: 44자 ✓
├─ 형식: RFC 5322? NO - "@" 위치 오류 ✗
└─ SQL 패턴 감지:
   Regex: \s+UNION\s+ → 매칭! ✓
   Result: Err(PossibleSQLInjection)

↓

HTTP 응답:
┌──────────────────────────────┐
│ HTTP/1.1 400 Bad Request     │
│                              │
│ input contains potentially   │
│ dangerous SQL patterns       │
└──────────────────────────────┘

↓

로깅:
{
  "level": "WARN",
  "message": "Invalid email received in subscription request",
  "error": "input contains potentially dangerous SQL patterns"
}
```

### 실패 케이스 2: 길이 초과

```
클라이언트 요청:
name=aaaa...aaaa (257자)

↓

Validators::is_valid_name()
├─ trim: (257자)
├─ 길이 확인: 257 > 256? YES ✗
└─ Result: Err(TooLong("name", 256))

↓

HTTP 응답:
┌──────────────────────────────┐
│ HTTP/1.1 400 Bad Request     │
│                              │
│ name is too long (maximum    │
│ 256 characters)              │
└──────────────────────────────┘
```

---

## 동시성 처리

### 멀티스레드 요청 처리

```
요청 1 (Thread A)          요청 2 (Thread B)
    │                          │
    ├─ john@example.com        ├─ jane@example.com
    │                          │
    ├─ 검증 ✓                  ├─ 검증 ✓
    │                          │
    ├─ DB 삽입 시작             ├─ DB 삽입 시작
    │  Mutex 획득               │  Mutex 대기
    │                          │
    ├─ INSERT (row 1)           │
    │                          │
    ├─ Mutex 해제              │
    │                          │  Mutex 획득
    │                          │
    │                          ├─ INSERT (row 2)
    │                          │
    │  ← 200 OK                ├─ Mutex 해제
    │                          │
    │                          ├─ 200 OK →
    │
결과: 두 이메일 모두 저장됨 (순서대로)
```

---

## 성능 특성

### 요청 처리 시간 분석

```
요청 처리 타임라인:

0ms    ┤ 요청 수신 (Actix-web)
1ms    ├─ FormData 파싱
2ms    ├─ 이메일 검증
       │  ├─ 길이: 0.1ms
       │  ├─ 형식: 0.1ms
       │  ├─ 패턴 (6개): 0.3ms
       │  └─ 피싱: 0.1ms
3.6ms  ├─ 이름 검증
       │  ├─ 길이: 0.1ms
       │  ├─ 패턴: 0.2ms
       │  └─ SQL: 0.3ms
4.2ms  ├─ UUID 생성
5.2ms  ├─ 데이터베이스 쿼리
       │  ├─ 준비: 0.5ms
       │  ├─ 실행: 3ms
       │  └─ 반환: 0.5ms
9.2ms  ├─ 응답 생성
9.5ms  └─ 로깅

총 시간: ~10ms per request
```

### 메모리 사용

```
고정 메모리:
├─ 정규표현식 (7개): ~50KB
├─ 설정 객체: <1KB
└─ 스레드 로컬 저장소: ~256KB per thread

요청당 추가 메모리:
├─ FormData: <1KB
├─ 검증 임시: <1KB
├─ DB 연결: 공유
└─ 응답: <1KB

총: ~1KB per request
```

---

## 확장성 고려사항

### 수평 확장 (Horizontal Scaling)

```
클라이언트 요청들
    │
    ├─→ Load Balancer
    │
    ├─→ Instance 1 (Port 8001)
    ├─→ Instance 2 (Port 8002)
    ├─→ Instance 3 (Port 8003)
    │
    └─→ PostgreSQL (공유)

각 인스턴스:
  • 독립적인 검증 로직
  • 공유 데이터베이스
  • Rate Limiting (인스턴스별)
```

### 데이터베이스 확장

```
현재: Single PostgreSQL
├─ 읽기/쓰기: 모두 main 데이터베이스
├─ 백업: 자동 WAL
└─ 용량: 충분

향후:
├─ Read Replicas: SELECT 분산
├─ Sharding: 데이터 분할
└─ Cache: Redis 캐시
```

---

## 모니터링 및 로깅

### 로그 흐름

```
요청
  │
  ├─→ LoggerMiddleware
  │   └─ 요청 정보 로깅
  │
  ├─→ Route Handler
  │   ├─ 검증 실패 → WARN 레벨
  │   ├─ DB 성공 → INFO 레벨
  │   └─ DB 실패 → ERROR 레벨
  │
  └─→ Tracing Subscriber
      └─ JSON 형식으로 저장
         (stdout, 파일, 또는 로깅 서비스)
```

### 메트릭 수집 (향후)

```
메트릭 포인트:
├─ 요청 수
│  └─ /subscriptions: N per second
│
├─ 응답 코드
│  ├─ 200 OK: X%
│  ├─ 400 Bad Request: Y%
│  ├─ 409 Conflict: Z%
│  └─ 500 Error: W%
│
├─ 검증 실패 이유
│  ├─ TooLong: A%
│  ├─ InvalidFormat: B%
│  └─ PossibleSQLInjection: C%
│
└─ 응답 시간
   ├─ 평균: Xms
   ├─ P50: Yms
   ├─ P95: Zms
   └─ P99: Wms
```

---

## 배포 아키텍처

### 개발 환경

```
Developer
    │
    ├─ 로컬 PostgreSQL
    ├─ cargo run
    └─ http://localhost:8002
```

### 스테이징 환경

```
GitHub (main branch)
    │
    ├─ CI/CD (GitHub Actions)
    │  ├─ cargo test
    │  ├─ cargo build
    │  └─ Docker build
    │
    └─ Docker Registry
       │
       └─ Staging Server
          ├─ PostgreSQL (staging)
          └─ Actix-web (docker)
```

### 프로덕션 환경

```
GitHub Release
    │
    ├─ Docker Image
    │  └─ Docker Registry
    │
    ├─ Kubernetes Cluster
    │  ├─ Pod 1: Actix-web
    │  ├─ Pod 2: Actix-web
    │  ├─ Pod 3: Actix-web
    │  │
    │  ├─ Load Balancer (Service)
    │  │
    │  └─ PostgreSQL StatefulSet
    │
    ├─ Monitoring (Prometheus, Grafana)
    │
    └─ Logging (ELK Stack, CloudWatch)
```

---

## 요약

이 문서에서 다룬 내용:
- **시스템 아키텍처**: 멀티레이어 구조
- **요청 흐름**: HTTP → Validation → DB
- **보안 흐름**: 다층 검증 체계
- **에러 처리**: 적절한 HTTP 상태 코드
- **성능**: ~10ms per request
- **확장성**: 수평/수직 확장 가능
- **모니터링**: 구조화된 로깅

---

**작성일**: 2025-11-20
**버전**: 1.0.0
