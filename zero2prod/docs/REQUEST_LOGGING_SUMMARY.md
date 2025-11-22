# 요청 실패 기록 시스템 구현 완료

## 📋 구현된 5가지 기능

### 1️⃣ 요청 메타데이터 기록 (RequestMetadata)
- **기능**: 모든 요청의 HTTP 메서드, 경로, 헤더, IP 주소 등 캡처
- **민감정보 보호**: Authorization, Cookie 등 자동 제외
- **생성 위치**: `src/request_logging.rs` (line 45-95)

```rust
RequestMetadata::new(request_id, "POST", "/subscriptions")
    .with_client_ip("192.168.1.100".to_string())
    .with_user_agent("Mozilla/5.0...")
```

### 2️⃣ 실패 요청 상세 기록 (FailedRequest)
- **기능**: 오류 유형, 메시지, HTTP 상태, 처리 시간, 재시도 정보 기록
- **오류 분류**: 일시적 오류, 클라이언트 오류, 서버 오류 자동 판별
- **생성 위치**: `src/request_logging.rs` (line 115-170)

```rust
FailedRequest::new(metadata, "ValidationError", "Invalid email", "VALIDATION_ERROR", 400)
    .with_error_details("Details...")
    .with_retryable(false)
```

### 3️⃣ 감사 로그 (Audit Trail)
- **기능**: 모든 작업의 성공/실패, 상태 변경 추적
- **용도**: 규제 준수, 보안 감사, 문제 조사
- **생성 위치**: `src/request_logging.rs` (line 245-300)

```rust
AuditLog::new("CREATE_SUBSCRIBER", "subscription", "SUCCESS", "...")
    .with_resource_id(subscriber_id.to_string())
    .with_state_change("pending", "confirmed")
```

### 4️⃣ 실패 요청 통계 (FailureStatistics)
- **기능**: 시간 단위로 실패율, 엔드포인트별 오류, 응답 시간 통계
- **용도**: 성능 모니터링, 문제 추적, 용량 계획
- **생성 위치**: `src/request_logging.rs` (line 310-380)

```rust
let mut stats = FailureStatistics::new(60);  // 60분 주기
for failed_request in &requests {
    stats.add_failure(&failed_request);
}
RequestFailureLogger::log_statistics(&stats);
```

### 5️⃣ RequestFailureLogger
- **기능**: 실패 요청, 감사 로그, 통계를 구조화된 JSON으로 기록
- **로깅 레벨**: WARN (검증, 중복), ERROR (시스템 오류)
- **생성 위치**: `src/request_logging.rs` (line 390-500)

```rust
RequestFailureLogger::log_failed_request(&failed_request);
RequestFailureLogger::log_audit(&audit_log);
RequestFailureLogger::log_statistics(&stats);
RequestFailureLogger::log_retry_attempt(&request, "reason");
RequestFailureLogger::log_retry_success(&request);
RequestFailureLogger::log_retry_exhausted(&request);
```

---

## 🔄 로깅 흐름

```
요청 도착
  ↓
[검증 단계]
  → 실패: AuditLog(VALIDATE_*) + WARN 로그
  → 성공: 다음 단계
  ↓
[데이터베이스 단계]
  → 실패: FailedRequest + AuditLog(CREATE_*) + ERROR 로그
  → 성공: 다음 단계
  ↓
[이메일 서비스]
  → 실패: FailedRequest(503, retryable=true) + AuditLog + ERROR 로그
  → 성공: AuditLog(SUCCESS)
  ↓
클라이언트에게 응답 반환
  ↓
로그 파일/ELK Stack/Datadog으로 전송
```

---

## 📊 로그 출력 예시

### 검증 실패
```json
{
    "level": "WARN",
    "message": "Audit log entry",
    "log_id": "audit-123",
    "action": "VALIDATE_EMAIL",
    "resource_type": "subscription",
    "status": "FAILURE",
    "message": "Email validation failed: email has invalid format",
    "timestamp": "2024-11-22T10:30:45.200Z"
}
```

### 데이터베이스 오류
```json
{
    "level": "ERROR",
    "message": "Failed request",
    "request_id": "req-456",
    "http_method": "POST",
    "request_path": "/subscriptions",
    "error_type": "DatabaseError",
    "error_code": "DUPLICATE_ENTRY",
    "response_status": 409,
    "duration_ms": 150,
    "is_retryable": "NO",
    "error_category": "CLIENT_ERROR"
}
```

### 이메일 서비스 오류 (재시도 가능)
```json
{
    "level": "ERROR",
    "message": "Failed request",
    "request_id": "req-789",
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

## 📁 구현 파일

### 새로 생성된 파일
1. **src/request_logging.rs** (600+ 줄)
   - RequestMetadata 구조
   - FailedRequest 구조  
   - AuditLog 구조
   - FailureStatistics 구조
   - RequestFailureLogger 클래스
   - 7개의 포괄적 테스트

2. **docs/REQUEST_FAILURE_LOGGING.md**
   - 상세한 한글 설명서
   - 실제 사용 예시
   - 로그 분석 방법
   - Elasticsearch/Grafana 설정

### 수정된 파일
1. **src/lib.rs**
   - request_logging 모듈 추가

2. **src/error.rs**
   - EmailError에 Clone 트레이트 추가

3. **src/routes/subscriptions.rs**
   - RequestMetadata 생성
   - 검증 단계에서 AuditLog 기록
   - 데이터베이스 오류 시 FailedRequest 기록
   - 이메일 오류 시 FailedRequest 기록
   - 성공 시 AuditLog 기록

---

## 🧪 테스트 결과

```
✅ 29개 테스트 모두 PASS
  - request_metadata_creation
  - request_metadata_sensitive_headers_excluded
  - failed_request_creation
  - failed_request_error_classification
  - audit_log_creation
  - failure_statistics
  - retry_count_increment
  (+ 더 많은 테스트)
```

---

## 📈 활용 시나리오

### 1. 실시간 모니터링
```
ElasticSearch ← 구조화된 JSON 로그 ← RequestFailureLogger
      ↓
  Kibana 대시보드에서 실시간 시각화
```

### 2. 문제 조사
```
request_id로 모든 관련 로그 추적
  ↓
타임라인 구성: 입력 검증 → DB 저장 → 이메일 전송
  ↓
어느 단계에서 실패했는지 파악
```

### 3. 오류 패턴 분석
```
failures_by_endpoint에서 문제 엔드포인트 식별
failures_by_type에서 공통 오류 찾기
duration_ms로 성능 문제 감지
```

### 4. 자동 재시도 구현
```
is_retryable=true인 오류만 자동 재시도
retry_count로 재시도 횟수 추적
last_retry_timestamp로 재시도 시간 기록
```

---

## 🎯 모니터링 규칙 (Prometheus)

```yaml
# 높은 실패율
alert: HighFailureRate
expr: rate(failures_total[5m]) > 0.05

# 이메일 서비스 장애
alert: EmailServiceDown
expr: failures_total{error_type="EmailError"} > 20

# 중복 항목 급증 (비즈니스 로직 문제)
alert: DuplicateEntries
expr: rate(failures_total{error_code="DUPLICATE_ENTRY"}[1h]) > 0.1
```

---

## 💾 로그 저장 전략

### 단기 (1일)
- 파일: `/var/log/application/*.log`
- 형식: JSON 라인 기반
- 로테이션: 일일

### 중기 (30일)
- ElasticSearch: 인덱싱된 저장
- Kibana: 시각화 대시보드
- 검색: request_id, error_type, timestamp로 쿼리

### 장기 (1년)
- S3/Archive: 압축 저장
- 규제 준수: 감사 로그 보관
- 법적 요구사항: 필요시 검색

---

## 🔐 보안 고려사항

### ✅ 구현된 보안 기능
1. **민감 헤더 자동 제외**
   - Authorization, Cookie, X-API-Key 등

2. **개인정보 보호**
   - 비밀번호는 로깅하지 않음
   - 이메일은 최소 정보만 기록

3. **감사 로그**
   - 누가, 언제, 무엇을 했는지 기록
   - 규제 준수 증거

### ⚠️ 고려사항
- 로그 저장소 암호화
- 로그 접근 제어
- 로그 보관 정책

---

## 📚 문서

1. **REQUEST_FAILURE_LOGGING.md**
   - 완전한 한글 가이드
   - 실제 사용 예시
   - ElasticSearch 쿼리

2. **소스 코드 주석**
   - request_logging.rs: 상세 설명
   - subscriptions.rs: 통합 예시

---

## 🚀 다음 단계 (선택사항)

1. **로그 저장소 연동**
   - ElasticSearch 통합
   - Logstash/Fluentd 설정

2. **대시보드 구성**
   - Kibana/Grafana
   - 실시간 모니터링

3. **알림 설정**
   - PagerDuty/OpsGenie
   - 중요 오류 알림

4. **자동 재시도**
   - 재시도 로직 구현
   - 지수 백오프 설정

---

## ✨ 주요 특징

| 기능 | 설명 |
|------|------|
| **요청 추적** | request_id로 전체 요청 흐름 추적 |
| **오류 분류** | 오류 타입별 자동 분류 및 카테고리화 |
| **재시도 관리** | 재시도 가능 여부 자동 판별 |
| **성능 모니터링** | 요청별 처리 시간 기록 |
| **감사 로그** | 모든 작업의 성공/실패 기록 |
| **통계 수집** | 시간 단위 오류 통계 |
| **구조화된 로깅** | JSON 형식으로 기계 가독성 보장 |
| **민감정보 보호** | 자동으로 민감한 정보 제외 |

---

## 📞 지원

문제 발생 시:
1. request_id로 모든 관련 로그 검색
2. error_type과 response_status 확인
3. 타임라인으로 실패 지점 파악
4. 로그 분석으로 원인 규명

---

## 🎉 완료!

모든 요청 실패는 이제 다음과 같이 기록됩니다:

✅ **요청 메타데이터** - 언제, 누가, 어디서, 무엇을 했는지
✅ **실패 상세** - 왜 실패했는지, 복구 가능한지
✅ **감사 로그** - 규제 준수, 보안 감시
✅ **통계** - 실시간 모니터링, 문제 분석
✅ **재시도 관리** - 자동 복구 가능성

프로덕션 환경에서 안정적이고 감사 가능한 오류 처리!

