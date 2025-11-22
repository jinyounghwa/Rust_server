/// 실패한 요청 상세 기록 시스템
///
/// 이 모듈은 다음을 담당합니다:
/// 1. 요청 메타데이터 기록 (HTTP 메서드, 경로, 헤더, 쿼리 파라미터)
/// 2. 오류 상세 분석
/// 3. 감사 로그 (Audit Trail)
/// 4. 실패 요청 통계
/// 5. 오류 복구 시도 로그

use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use std::collections::HashMap;
use uuid::Uuid;

// DateTime을 직렬화 가능하게 하기 위한 모듈
mod datetime_format {
    use chrono::{DateTime, Utc};
    use serde::{Deserialize, Deserializer, Serializer};

    pub fn serialize<S>(dt: &DateTime<Utc>, ser: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        ser.serialize_str(&dt.to_rfc3339())
    }

    pub fn deserialize<'de, D>(deser: D) -> Result<DateTime<Utc>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deser)?;
        DateTime::parse_from_rfc3339(&s)
            .map(|dt| dt.with_timezone(&Utc))
            .map_err(serde::de::Error::custom)
    }
}

// Option<DateTime>을 직렬화 가능하게 하기 위한 모듈
mod option_datetime_format {
    use chrono::{DateTime, Utc};
    use serde::{Deserialize, Deserializer, Serializer};

    pub fn serialize<S>(dt: &Option<DateTime<Utc>>, ser: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match dt {
            Some(dt) => ser.serialize_str(&dt.to_rfc3339()),
            None => ser.serialize_none(),
        }
    }

    pub fn deserialize<'de, D>(deser: D) -> Result<Option<DateTime<Utc>>, D::Error>
    where
        D: Deserializer<'de>,
    {
        Option::<String>::deserialize(deser)?
            .map(|s| {
                DateTime::parse_from_rfc3339(&s)
                    .map(|dt| dt.with_timezone(&Utc))
                    .map_err(serde::de::Error::custom)
            })
            .transpose()
    }
}

/// ============================================================================
/// 1. 요청 메타데이터 구조
/// ============================================================================

/// HTTP 요청 메타데이터
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequestMetadata {
    /// 고유 요청 ID
    pub request_id: String,
    /// HTTP 메서드 (GET, POST, PUT, DELETE 등)
    pub http_method: String,
    /// 요청 경로
    pub request_path: String,
    /// 쿼리 파라미터
    pub query_params: HashMap<String, String>,
    /// 요청 헤더 (민감한 정보 제외)
    pub headers: HashMap<String, String>,
    /// 클라이언트 IP 주소
    pub client_ip: Option<String>,
    /// 요청 타임스탐프
    #[serde(with = "datetime_format")]
    pub request_timestamp: DateTime<Utc>,
    /// User-Agent
    pub user_agent: Option<String>,
    /// 사용자 ID (해당하는 경우)
    pub user_id: Option<String>,
}

impl RequestMetadata {
    pub fn new(request_id: String, http_method: String, request_path: String) -> Self {
        Self {
            request_id,
            http_method,
            request_path,
            query_params: HashMap::new(),
            headers: HashMap::new(),
            client_ip: None,
            request_timestamp: Utc::now(),
            user_agent: None,
            user_id: None,
        }
    }

    pub fn with_client_ip(mut self, ip: String) -> Self {
        self.client_ip = Some(ip);
        self
    }

    pub fn with_user_agent(mut self, ua: String) -> Self {
        self.user_agent = Some(ua);
        self
    }

    pub fn with_user_id(mut self, user_id: String) -> Self {
        self.user_id = Some(user_id);
        self
    }

    pub fn with_query_params(mut self, params: HashMap<String, String>) -> Self {
        self.query_params = params;
        self
    }

    pub fn add_header(mut self, key: String, value: String) -> Self {
        // 민감한 헤더 제외
        let sensitive_headers = ["authorization", "cookie", "x-api-key", "x-token"];
        if !sensitive_headers.contains(&key.to_lowercase().as_str()) {
            self.headers.insert(key, value);
        }
        self
    }
}

/// ============================================================================
/// 2. 실패 요청 기록
/// ============================================================================

/// 실패 요청의 상세 정보
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FailedRequest {
    /// 요청 메타데이터
    pub request_metadata: RequestMetadata,
    /// 오류 타입
    pub error_type: String,
    /// 오류 메시지
    pub error_message: String,
    /// 오류 코드
    pub error_code: String,
    /// HTTP 응답 상태 코드
    pub response_status: u16,
    /// 응답 타임스탐프
    #[serde(with = "datetime_format")]
    pub response_timestamp: DateTime<Utc>,
    /// 요청 처리 시간 (밀리초)
    pub duration_ms: u64,
    /// 오류 상세 정보 (스택 트레이스 등)
    pub error_details: Option<String>,
    /// 재시도 가능 여부
    pub is_retryable: bool,
    /// 재시도 횟수
    pub retry_count: u32,
    /// 마지막 재시도 시각
    #[serde(with = "option_datetime_format")]
    pub last_retry_timestamp: Option<DateTime<Utc>>,
}

impl FailedRequest {
    pub fn new(
        request_metadata: RequestMetadata,
        error_type: String,
        error_message: String,
        error_code: String,
        response_status: u16,
    ) -> Self {
        let response_timestamp = Utc::now();
        let duration_ms = response_timestamp
            .signed_duration_since(request_metadata.request_timestamp)
            .num_milliseconds() as u64;

        Self {
            request_metadata,
            error_type,
            error_message,
            error_code,
            response_status,
            response_timestamp,
            duration_ms,
            error_details: None,
            is_retryable: false,
            retry_count: 0,
            last_retry_timestamp: None,
        }
    }

    pub fn with_error_details(mut self, details: String) -> Self {
        self.error_details = Some(details);
        self
    }

    pub fn with_retryable(mut self, retryable: bool) -> Self {
        self.is_retryable = retryable;
        self
    }

    pub fn increment_retry_count(&mut self) {
        self.retry_count += 1;
        self.last_retry_timestamp = Some(Utc::now());
    }

    /// 오류가 일시적 오류인지 확인 (재시도 가능)
    pub fn is_temporary_error(&self) -> bool {
        matches!(
            self.response_status,
            408 | 429 | 500 | 502 | 503 | 504 // Request Timeout, Too Many Requests, Server errors
        )
    }

    /// 오류가 클라이언트 오류인지 확인 (재시도 불가)
    pub fn is_client_error(&self) -> bool {
        self.response_status >= 400 && self.response_status < 500
    }

    /// 오류가 서버 오류인지 확인
    pub fn is_server_error(&self) -> bool {
        self.response_status >= 500
    }
}

/// ============================================================================
/// 3. 감사 로그 (Audit Trail)
/// ============================================================================

/// 감사 로그 항목
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditLog {
    /// 로그 ID
    pub log_id: String,
    /// 타임스탐프
    #[serde(with = "datetime_format")]
    pub timestamp: DateTime<Utc>,
    /// 작업 유형 (CREATE, READ, UPDATE, DELETE, ERROR 등)
    pub action: String,
    /// 리소스 유형 (subscription, user, token 등)
    pub resource_type: String,
    /// 리소스 ID
    pub resource_id: Option<String>,
    /// 사용자 ID
    pub user_id: Option<String>,
    /// 상태 (SUCCESS, FAILURE)
    pub status: String,
    /// 상세 메시지
    pub message: String,
    /// 변경 전 상태 (해당하는 경우)
    pub previous_state: Option<String>,
    /// 변경 후 상태 (해당하는 경우)
    pub new_state: Option<String>,
}

impl AuditLog {
    pub fn new(action: String, resource_type: String, status: String, message: String) -> Self {
        Self {
            log_id: Uuid::new_v4().to_string(),
            timestamp: Utc::now(),
            action,
            resource_type,
            resource_id: None,
            user_id: None,
            status,
            message,
            previous_state: None,
            new_state: None,
        }
    }

    pub fn with_resource_id(mut self, id: String) -> Self {
        self.resource_id = Some(id);
        self
    }

    pub fn with_user_id(mut self, user_id: String) -> Self {
        self.user_id = Some(user_id);
        self
    }

    pub fn with_state_change(mut self, previous: String, new: String) -> Self {
        self.previous_state = Some(previous);
        self.new_state = Some(new);
        self
    }
}

/// ============================================================================
/// 4. 실패 요청 통계
/// ============================================================================

/// 실패 요청 통계
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FailureStatistics {
    /// 통계 기간 (분)
    pub period_minutes: u32,
    /// 총 실패 요청 수
    pub total_failures: u32,
    /// 오류 타입별 카운트
    pub failures_by_type: HashMap<String, u32>,
    /// 엔드포인트별 실패 카운트
    pub failures_by_endpoint: HashMap<String, u32>,
    /// HTTP 상태 코드별 카운트
    pub failures_by_status: HashMap<u16, u32>,
    /// 재시도 가능한 오류 수
    pub retryable_errors: u32,
    /// 평균 응답 시간 (밀리초)
    pub average_response_time_ms: u64,
    /// 최장 응답 시간 (밀리초)
    pub max_response_time_ms: u64,
    /// 최단 응답 시간 (밀리초)
    pub min_response_time_ms: u64,
}

impl Default for FailureStatistics {
    fn default() -> Self {
        Self {
            period_minutes: 60,
            total_failures: 0,
            failures_by_type: HashMap::new(),
            failures_by_endpoint: HashMap::new(),
            failures_by_status: HashMap::new(),
            retryable_errors: 0,
            average_response_time_ms: 0,
            max_response_time_ms: 0,
            min_response_time_ms: u64::MAX,
        }
    }
}

impl FailureStatistics {
    pub fn new(period_minutes: u32) -> Self {
        Self {
            period_minutes,
            ..Default::default()
        }
    }

    /// 실패 요청 추가
    pub fn add_failure(&mut self, failed_request: &FailedRequest) {
        self.total_failures += 1;

        // 오류 타입별 카운트
        *self
            .failures_by_type
            .entry(failed_request.error_type.clone())
            .or_insert(0) += 1;

        // 엔드포인트별 실패 카운트
        *self
            .failures_by_endpoint
            .entry(failed_request.request_metadata.request_path.clone())
            .or_insert(0) += 1;

        // HTTP 상태 코드별 카운트
        *self
            .failures_by_status
            .entry(failed_request.response_status)
            .or_insert(0) += 1;

        // 재시도 가능한 오류 수
        if failed_request.is_retryable {
            self.retryable_errors += 1;
        }

        // 응답 시간 통계
        self.update_response_time_stats(failed_request.duration_ms);
    }

    fn update_response_time_stats(&mut self, duration_ms: u64) {
        // 최대값 업데이트
        if duration_ms > self.max_response_time_ms {
            self.max_response_time_ms = duration_ms;
        }

        // 최소값 업데이트
        if duration_ms < self.min_response_time_ms {
            self.min_response_time_ms = duration_ms;
        }

        // 평균값 업데이트 (단순 계산)
        if self.total_failures > 0 {
            self.average_response_time_ms =
                (self.average_response_time_ms + duration_ms) / 2;
        }
    }

    /// 통계 요약
    pub fn summary(&self) -> String {
        format!(
            "Failure Statistics (last {} minutes): Total: {}, Retryable: {}, Avg Response: {}ms",
            self.period_minutes,
            self.total_failures,
            self.retryable_errors,
            self.average_response_time_ms
        )
    }
}

/// ============================================================================
/// 5. 요청 로거 (Request Logger)
/// ============================================================================

/// 요청 실패 로거
pub struct RequestFailureLogger;

impl RequestFailureLogger {
    /// 상세 오류 로그 기록
    pub fn log_failed_request(failed_request: &FailedRequest) {
        let is_retryable = if failed_request.is_retryable { "YES" } else { "NO" };
        let is_temporary = if failed_request.is_temporary_error() {
            "TEMPORARY"
        } else if failed_request.is_client_error() {
            "CLIENT_ERROR"
        } else {
            "SERVER_ERROR"
        };

        tracing::error!(
            request_id = %failed_request.request_metadata.request_id,
            http_method = %failed_request.request_metadata.http_method,
            request_path = %failed_request.request_metadata.request_path,
            client_ip = ?failed_request.request_metadata.client_ip,
            user_id = ?failed_request.request_metadata.user_id,
            error_type = %failed_request.error_type,
            error_code = %failed_request.error_code,
            response_status = failed_request.response_status,
            duration_ms = failed_request.duration_ms,
            is_retryable = is_retryable,
            error_category = is_temporary,
            retry_count = failed_request.retry_count,
            "Failed request",
        );

        // 오류 메시지 로그
        tracing::error!(
            request_id = %failed_request.request_metadata.request_id,
            error = %failed_request.error_message,
            "Error message"
        );

        // 상세 정보가 있으면 로그
        if let Some(details) = &failed_request.error_details {
            tracing::error!(
                request_id = %failed_request.request_metadata.request_id,
                details = %details,
                "Error details"
            );
        }
    }

    /// 감사 로그 기록
    pub fn log_audit(audit_log: &AuditLog) {
        if audit_log.status == "FAILURE" {
            tracing::warn!(
                log_id = %audit_log.log_id,
                action = %audit_log.action,
                resource_type = %audit_log.resource_type,
                resource_id = ?audit_log.resource_id,
                user_id = ?audit_log.user_id,
                status = %audit_log.status,
                message = %audit_log.message,
                "Audit log entry"
            );
        } else {
            tracing::info!(
                log_id = %audit_log.log_id,
                action = %audit_log.action,
                resource_type = %audit_log.resource_type,
                resource_id = ?audit_log.resource_id,
                user_id = ?audit_log.user_id,
                status = %audit_log.status,
                message = %audit_log.message,
                "Audit log entry"
            );
        }
    }

    /// 실패 요청 통계 로그
    pub fn log_statistics(stats: &FailureStatistics) {
        tracing::warn!(
            total_failures = stats.total_failures,
            retryable_errors = stats.retryable_errors,
            average_response_time_ms = stats.average_response_time_ms,
            max_response_time_ms = stats.max_response_time_ms,
            min_response_time_ms = stats.min_response_time_ms,
            "{}", stats.summary()
        );

        // 오류 타입별 분포
        for (error_type, count) in &stats.failures_by_type {
            tracing::warn!(
                error_type = %error_type,
                count = count,
                "Error type distribution"
            );
        }

        // 엔드포인트별 실패율
        for (endpoint, count) in &stats.failures_by_endpoint {
            tracing::warn!(
                endpoint = %endpoint,
                failure_count = count,
                "Failures by endpoint"
            );
        }

        // HTTP 상태 코드별 분포
        for (status, count) in &stats.failures_by_status {
            tracing::warn!(
                status = status,
                count = count,
                "Failures by HTTP status"
            );
        }
    }

    /// 재시도 시도 로그
    pub fn log_retry_attempt(failed_request: &FailedRequest, reason: &str) {
        tracing::info!(
            request_id = %failed_request.request_metadata.request_id,
            retry_count = failed_request.retry_count,
            reason = %reason,
            "Retrying failed request"
        );
    }

    /// 재시도 성공 로그
    pub fn log_retry_success(failed_request: &FailedRequest) {
        tracing::info!(
            request_id = %failed_request.request_metadata.request_id,
            original_retry_count = failed_request.retry_count,
            "Retry succeeded after previous failure"
        );
    }

    /// 재시도 최종 실패 로그
    pub fn log_retry_exhausted(failed_request: &FailedRequest) {
        tracing::error!(
            request_id = %failed_request.request_metadata.request_id,
            error_type = %failed_request.error_type,
            error_code = %failed_request.error_code,
            total_retry_count = failed_request.retry_count,
            "Retry exhausted, giving up"
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_request_metadata_creation() {
        let metadata = RequestMetadata::new(
            "test-123".to_string(),
            "POST".to_string(),
            "/subscriptions".to_string(),
        );

        assert_eq!(metadata.request_id, "test-123");
        assert_eq!(metadata.http_method, "POST");
        assert_eq!(metadata.request_path, "/subscriptions");
    }

    #[test]
    fn test_request_metadata_sensitive_headers_excluded() {
        let metadata = RequestMetadata::new(
            "test-123".to_string(),
            "POST".to_string(),
            "/subscriptions".to_string(),
        )
        .add_header("Content-Type".to_string(), "application/json".to_string())
        .add_header("Authorization".to_string(), "Bearer secret".to_string());

        assert!(metadata.headers.contains_key("Content-Type"));
        assert!(!metadata.headers.contains_key("Authorization"));
    }

    #[test]
    fn test_failed_request_creation() {
        let metadata = RequestMetadata::new(
            "test-123".to_string(),
            "POST".to_string(),
            "/subscriptions".to_string(),
        );

        let failed_request = FailedRequest::new(
            metadata,
            "ValidationError".to_string(),
            "Email is invalid".to_string(),
            "VALIDATION_ERROR".to_string(),
            400,
        );

        assert_eq!(failed_request.response_status, 400);
        assert!(failed_request.is_client_error());
    }

    #[test]
    fn test_failed_request_error_classification() {
        let metadata = RequestMetadata::new(
            "test-123".to_string(),
            "GET".to_string(),
            "/data".to_string(),
        );

        // 일시적 오류
        let temp_error = FailedRequest::new(
            metadata.clone(),
            "ServiceUnavailable".to_string(),
            "Service temporarily unavailable".to_string(),
            "SERVICE_UNAVAILABLE".to_string(),
            503,
        );
        assert!(temp_error.is_temporary_error());

        // 클라이언트 오류
        let client_error = FailedRequest::new(
            metadata.clone(),
            "ValidationError".to_string(),
            "Invalid input".to_string(),
            "VALIDATION_ERROR".to_string(),
            400,
        );
        assert!(client_error.is_client_error());

        // 서버 오류
        let server_error = FailedRequest::new(
            metadata,
            "InternalError".to_string(),
            "Internal server error".to_string(),
            "INTERNAL_ERROR".to_string(),
            500,
        );
        assert!(server_error.is_server_error());
    }

    #[test]
    fn test_audit_log_creation() {
        let audit_log = AuditLog::new(
            "CREATE".to_string(),
            "subscription".to_string(),
            "FAILURE".to_string(),
            "Failed to create subscription".to_string(),
        )
        .with_resource_id("sub-123".to_string())
        .with_user_id("user-456".to_string());

        assert_eq!(audit_log.action, "CREATE");
        assert_eq!(audit_log.resource_type, "subscription");
        assert_eq!(audit_log.resource_id, Some("sub-123".to_string()));
    }

    #[test]
    fn test_failure_statistics() {
        let mut stats = FailureStatistics::new(60);

        let metadata = RequestMetadata::new(
            "test-123".to_string(),
            "POST".to_string(),
            "/subscriptions".to_string(),
        );

        let failed_request = FailedRequest::new(
            metadata,
            "ValidationError".to_string(),
            "Email is invalid".to_string(),
            "VALIDATION_ERROR".to_string(),
            400,
        )
        .with_retryable(false);

        stats.add_failure(&failed_request);

        assert_eq!(stats.total_failures, 1);
        assert_eq!(stats.retryable_errors, 0);
        assert!(stats.failures_by_type.contains_key("ValidationError"));
    }

    #[test]
    fn test_retry_count_increment() {
        let metadata = RequestMetadata::new(
            "test-123".to_string(),
            "GET".to_string(),
            "/data".to_string(),
        );

        let mut failed_request = FailedRequest::new(
            metadata,
            "ServiceUnavailable".to_string(),
            "Service unavailable".to_string(),
            "SERVICE_UNAVAILABLE".to_string(),
            503,
        );

        assert_eq!(failed_request.retry_count, 0);

        failed_request.increment_retry_count();
        assert_eq!(failed_request.retry_count, 1);
        assert!(failed_request.last_retry_timestamp.is_some());

        failed_request.increment_retry_count();
        assert_eq!(failed_request.retry_count, 2);
    }
}
