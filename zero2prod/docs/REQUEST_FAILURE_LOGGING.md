# 요청 실패 기록 시스템 (Request Failure Logging System)

## 개요

이 문서는 요청이 실패했을 때 이를 상세히 기록하는 완전한 시스템을 설명합니다.

## 📊 시스템 구성

```
요청 수신
  ↓
[검증 처리] → 검증 실패 → 감사 로그 (FAILURE)
  ↓
[데이터베이스] → DB 오류 → 실패 요청 기록 + 감사 로그
  ↓
[이메일 서비스] → 이메일 오류 → 실패 요청 기록 + 감사 로그
  ↓
성공 → 감사 로그 (SUCCESS) ✅
  ↓
구조화된 JSON 로그
  ↓
로그 파일 / ELK / 모니터링 시스템
```

---

## 1️⃣ 요청 메타데이터 기록 (RequestMetadata)

### 기능

모든 요청의 메타데이터를 캡처합니다:

```rust
pub struct RequestMetadata {
    pub request_id: String,              // 고유 ID
    pub http_method: String,             // GET, POST, PUT, DELETE
    pub request_path: String,            // /subscriptions, /health_check 등
    pub query_params: HashMap<...>,      // 쿼리 파라미터
    pub headers: HashMap<...>,           // HTTP 헤더 (민감정보 제외)
    pub client_ip: Option<String>,       // 클라이언트 IP
    pub request_timestamp: DateTime<Utc>, // 요청 시각
    pub user_agent: Option<String>,      // User-Agent
    pub user_id: Option<String>,         // 사용자 ID
}
```

### 사용 예시

```rust
let request_metadata = RequestMetadata::new(
    request_id.clone(),
    "POST".to_string(),
    "/subscriptions".to_string(),
)
.with_client_ip("192.168.1.100".to_string())
.with_user_agent("Mozilla/5.0...".to_string())
.add_header("Content-Type".to_string(), "application/json".to_string());
// Authorization 헤더는 자동으로 제외됨 (보안)
```

### 민감한 헤더 자동 제외

다음 헤더는 자동으로 제외됩니다:
- `Authorization`
- `Cookie`
- `X-API-Key`
- `X-Token`

---

## 2️⃣ 실패 요청 상세 기록 (FailedRequest)

### 기능

오류가 발생한 요청의 모든 정보를 기록합니다:

```rust
pub struct FailedRequest {
    pub request_metadata: RequestMetadata,    // 요청 정보
    pub error_type: String,                   // ValidationError, DatabaseError 등
    pub error_message: String,                // 사용자 친화적 메시지
    pub error_code: String,                   // VALIDATION_ERROR, DUPLICATE_ENTRY 등
    pub response_status: u16,                 // 400, 409, 500 등
    pub response_timestamp: DateTime<Utc>,    // 응답 시각
    pub duration_ms: u64,                     // 처리 시간 (밀리초)
    pub error_details: Option<String>,        // 상세 정보
    pub is_retryable: bool,                   // 재시도 가능 여부
    pub retry_count: u32,                     // 재시도 횟수
    pub last_retry_timestamp: Option<DateTime<Utc>>,  // 마지막 재시도 시각
}
```

### 사용 예시

```rust
let failed_request = FailedRequest::new(
    request_metadata,
    "ValidationError".to_string(),
    "Email is invalid".to_string(),
    "VALIDATION_ERROR".to_string(),
    400,  // HTTP 상태 코드
)
.with_error_details("Email format does not match RFC 5322".to_string())
.with_retryable(false);

// 실패 기록
RequestFailureLogger::log_failed_request(&failed_request);
```

### 오류 분류

```rust
// 일시적 오류 (재시도 가능)
failed_request.is_temporary_error()  // 408, 429, 500-504

// 클라이언트 오류 (재시도 불가)
failed_request.is_client_error()     // 400-499

// 서버 오류
failed_request.is_server_error()     // 500-599
```

---

## 3️⃣ 감사 로그 (Audit Trail)

### 기능

모든 작업의 성공/실패를 추적합니다:

```rust
pub struct AuditLog {
    pub log_id: String,                    // 고유 로그 ID
    pub timestamp: DateTime<Utc>,          // 타임스탐프
    pub action: String,                    // CREATE, READ, UPDATE, DELETE, VALIDATE
    pub resource_type: String,             // subscription, email, token
    pub resource_id: Option<String>,       // 리소스 ID
    pub user_id: Option<String>,           // 사용자 ID
    pub status: String,                    // SUCCESS or FAILURE
    pub message: String,                   // 상세 메시지
    pub previous_state: Option<String>,    // 변경 전 상태
    pub new_state: Option<String>,         // 변경 후 상태
}
```

### 사용 예시

#### 검증 실패

```rust
let audit_log = AuditLog::new(
    "VALIDATE_EMAIL".to_string(),
    "subscription".to_string(),
    "FAILURE".to_string(),
    format!("Email validation failed: {}", e),
)
.with_resource_id("sub-123".to_string());

RequestFailureLogger::log_audit(&audit_log);
```

#### 성공

```rust
let audit_log = AuditLog::new(
    "CREATE_SUBSCRIBER".to_string(),
    "subscription".to_string(),
    "SUCCESS".to_string(),
    "Subscriber created successfully".to_string(),
)
.with_resource_id(subscriber_id.to_string());

RequestFailureLogger::log_audit(&audit_log);
```

#### 상태 변경 추적

```rust
let audit_log = AuditLog::new(
    "UPDATE_STATUS".to_string(),
    "subscription".to_string(),
    "SUCCESS".to_string(),
    "Status updated".to_string(),
)
.with_resource_id(subscription_id.to_string())
.with_state_change("pending".to_string(), "confirmed".to_string());

RequestFailureLogger::log_audit(&audit_log);
```

---

## 4️⃣ 실패 요청 통계 (FailureStatistics)

### 기능

시간 단위로 실패 통계를 집계합니다:

```rust
pub struct FailureStatistics {
    pub period_minutes: u32,                      // 통계 기간
    pub total_failures: u32,                      // 총 실패 수
    pub failures_by_type: HashMap<String, u32>,   // 오류 타입별 카운트
    pub failures_by_endpoint: HashMap<String, u32>, // 엔드포인트별 카운트
    pub failures_by_status: HashMap<u16, u32>,    // HTTP 상태 코드별
    pub retryable_errors: u32,                    // 재시도 가능 오류
    pub average_response_time_ms: u64,            // 평균 응답 시간
    pub max_response_time_ms: u64,                // 최대 응답 시간
    pub min_response_time_ms: u64,                // 최소 응답 시간
}
```

### 사용 예시

```rust
let mut stats = FailureStatistics::new(60);  // 60분 주기

for failed_request in &failed_requests {
    stats.add_failure(failed_request);
}

// 통계 로그
RequestFailureLogger::log_statistics(&stats);

// 출력:
// Failure Statistics (last 60 minutes):
//   Total: 42,
//   Retryable: 15,
//   Avg Response: 245ms
```

---

## 5️⃣ 구조화된 로깅 출력

### 검증 오류 로그

```json
{
    "timestamp": "2024-11-22T10:30:45.123Z",
    "level": "WARN",
    "message": "Failed request",
    "request_id": "123e4567-e89b-12d3-a456-426614174000",
    "http_method": "POST",
    "request_path": "/subscriptions",
    "client_ip": "192.168.1.100",
    "user_id": null,
    "error_type": "ValidationError",
    "error_code": "VALIDATION_ERROR",
    "response_status": 400,
    "duration_ms": 5,
    "is_retryable": "NO",
    "error_category": "CLIENT_ERROR",
    "retry_count": 0
}
```

### 감사 로그

```json
{
    "timestamp": "2024-11-22T10:30:45.200Z",
    "level": "WARN",
    "log_id": "audit-456",
    "action": "VALIDATE_EMAIL",
    "resource_type": "subscription",
    "resource_id": null,
    "user_id": null,
    "status": "FAILURE",
    "message": "Email validation failed: email has invalid format",
    "previous_state": null,
    "new_state": null
}
```

### 데이터베이스 오류 로그

```json
{
    "timestamp": "2024-11-22T10:30:46.300Z",
    "level": "ERROR",
    "message": "Failed request",
    "request_id": "123e4567-e89b-12d3-a456-426614174001",
    "http_method": "POST",
    "request_path": "/subscriptions",
    "error_type": "DatabaseError",
    "error_code": "DUPLICATE_ENTRY",
    "response_status": 409,
    "duration_ms": 150,
    "is_retryable": "NO",
    "error_category": "CLIENT_ERROR",
    "retry_count": 0
}
```

### 이메일 오류 로그

```json
{
    "timestamp": "2024-11-22T10:30:47.400Z",
    "level": "ERROR",
    "message": "Failed request",
    "request_id": "123e4567-e89b-12d3-a456-426614174002",
    "http_method": "POST",
    "request_path": "/subscriptions",
    "error_type": "EmailError",
    "error_code": "EMAIL_SERVICE_ERROR",
    "response_status": 503,
    "duration_ms": 3000,
    "is_retryable": "YES",
    "error_category": "TEMPORARY",
    "retry_count": 0
}
```

---

## 6️⃣ 실제 구현 예시

### subscriptions.rs에서의 사용

```rust
pub async fn subscribe(
    form: web::Form<FormData>,
    pool: web::Data<PgPool>,
    email_client: web::Data<EmailClient>,
) -> Result<HttpResponse, AppError> {
    let error_context = ErrorContext::new("subscription_creation");

    // 1. 검증 및 감시 로깅
    let email = form.email.as_ref()
        .ok_or_else(|| {
            let audit_log = AuditLog::new(
                "VALIDATE_INPUT".to_string(),
                "subscription".to_string(),
                "FAILURE".to_string(),
                "Missing required field: email".to_string(),
            );
            RequestFailureLogger::log_audit(&audit_log);

            AppError::Validation(...)
        })?;

    // 2. 이메일 검증
    let email = is_valid_email(email)
        .map_err(|e| {
            let audit_log = AuditLog::new(
                "VALIDATE_EMAIL".to_string(),
                "subscription".to_string(),
                "FAILURE".to_string(),
                format!("Email validation failed: {}", e),
            );
            RequestFailureLogger::log_audit(&audit_log);
            AppError::Validation(e)
        })?;

    // 3. 데이터베이스 저장
    create_subscriber(&pool, subscriber_id, &email, &name, &error_context)
        .await
        .map_err(|e| {
            // 이미 create_subscriber 내부에서 로깅됨
            e
        })?;

    // 4. 이메일 전송
    send_confirmation_email_flow(
        email_client.get_ref(),
        &email,
        &name,
        &confirmation_token,
        &error_context,
    )
    .await
    .map_err(|e| {
        // 이미 send_confirmation_email_flow 내부에서 로깅됨
        e
    })?;

    // 5. 성공 로그
    let audit_log = AuditLog::new(
        "CREATE_SUBSCRIPTION".to_string(),
        "subscription".to_string(),
        "SUCCESS".to_string(),
        "Subscription created successfully".to_string(),
    )
    .with_resource_id(subscriber_id.to_string());
    RequestFailureLogger::log_audit(&audit_log);

    Ok(HttpResponse::Ok().finish())
}
```

---

## 📈 로그 분석 예시

### ElasticSearch에서 쿼리

```elasticsearch
# 1시간 내 모든 실패 요청
GET /logs-*/_search
{
  "query": {
    "bool": {
      "must": [
        { "match": { "level": "ERROR" } },
        { "range": { "timestamp": { "gte": "now-1h" } } }
      ]
    }
  }
}

# 특정 요청 ID의 모든 로그 추적
GET /logs-*/_search
{
  "query": { "match": { "request_id": "123e4567-e89b-12d3-a456-426614174000" } }
}

# 엔드포인트별 실패율
GET /logs-*/_search
{
  "aggs": {
    "by_endpoint": {
      "terms": { "field": "request_path.keyword" },
      "aggs": {
        "failures": { "filter": { "term": { "level": "ERROR" } } }
      }
    }
  }
}

# 오류 타입별 분포
GET /logs-*/_search
{
  "aggs": {
    "by_error_type": {
      "terms": { "field": "error_type.keyword" }
    }
  }
}
```

### Grafana 대시보드 구성

```yaml
# 1. 시간별 실패 요청 수
SELECT COUNT(*) FROM logs WHERE level='ERROR' GROUP BY time(1m)

# 2. 엔드포인트별 실패율
SELECT request_path, COUNT(*) as failures
FROM logs
WHERE level='ERROR'
GROUP BY request_path

# 3. 평균 응답 시간 추이
SELECT AVG(duration_ms) FROM logs GROUP BY time(5m)

# 4. 재시도 가능한 오류 비율
SELECT COUNT(*) FROM logs
WHERE is_retryable='YES' and level='ERROR'
```

---

## 🔍 모니터링 및 알림

### 알림 규칙 (Prometheus)

```yaml
# 높은 실패율 감지
alert: HighFailureRate
expr: rate(failures_total[5m]) > 0.05
annotations:
  summary: "Failure rate > 5% in last 5 minutes"

# 특정 엔드포인트 실패
alert: EndpointFailure
expr: increase(failures_total{endpoint="/subscriptions"}[5m]) > 10
annotations:
  summary: "More than 10 failures on /subscriptions in 5 minutes"

# 이메일 서비스 장애
alert: EmailServiceDown
expr: failures_total{error_type="EmailError"} > 20
annotations:
  summary: "Email service appears to be down"
```

---

## ✅ 체크리스트

### 새로운 엔드포인트 추가 시

- [ ] RequestMetadata 생성
- [ ] 검증 실패 시 AuditLog 기록
- [ ] 데이터베이스 오류 시 FailedRequest 기록
- [ ] 외부 서비스 오류 시 FailedRequest 기록
- [ ] 성공 시 AuditLog 기록
- [ ] 오류 분류 (TEMPORARY/CLIENT_ERROR/SERVER_ERROR) 설정
- [ ] 재시도 가능 여부 설정

### 로그 분석

- [ ] 일일 실패율 리뷰
- [ ] 재시도 불가능한 오류 조사
- [ ] 응답 시간 추이 모니터링
- [ ] 엔드포인트별 문제 해결

---

## 🎯 베스트 프랙티스

1. **민감한 데이터 보호**
   - 비밀번호, 토큰 등은 로깅하지 않기
   - 헤더에서 민감 정보 자동 제외

2. **구조화된 로깅**
   - JSON 형식으로 로그 저장
   - 기계 가독성 중시

3. **요청 추적**
   - 모든 요청에 고유 ID 부여
   - 관련 로그를 request_id로 추적

4. **효율적인 로그 관리**
   - 로그 레벨 적절히 설정 (WARN/ERROR)
   - 중요하지 않은 로그는 제한

5. **실시간 모니터링**
   - 임계값 설정 및 알림 구성
   - 중요 오류는 즉시 알림

---

## 참고 자료

- `src/request_logging.rs` - 핵심 구현
- `src/routes/subscriptions.rs` - 실제 사용 예시
- 로그 집계: ELK Stack, Splunk, Datadog 등
