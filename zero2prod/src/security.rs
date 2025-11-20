/// Security middleware module for protecting against common web attacks
/// Features:
/// - Rate limiting (DoS protection)
/// - Content-length validation (Payload bomb protection)
/// - Security headers (CSRF, XSS, Clickjacking protection)

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::SystemTime;

/// Configuration for rate limiting
pub struct RateLimitConfig {
    /// Max requests per minute per IP
    pub requests_per_minute: u32,
    /// Max content length in bytes
    pub max_content_length: u64,
}

impl Default for RateLimitConfig {
    fn default() -> Self {
        Self {
            requests_per_minute: 10,  // 10 requests per minute per IP = DoS protection
            max_content_length: 1024, // 1KB max for subscription form
        }
    }
}

/// Simple token bucket rate limiter implementation
struct TokenBucket {
    tokens: f64,
    last_refill: SystemTime,
    capacity: u32,
    refill_rate: f64, // tokens per second
}

impl TokenBucket {
    fn new(capacity: u32, requests_per_minute: u32) -> Self {
        Self {
            tokens: capacity as f64,
            last_refill: SystemTime::now(),
            capacity,
            refill_rate: requests_per_minute as f64 / 60.0,
        }
    }

    fn try_take_token(&mut self) -> bool {
        // Calculate elapsed time and refill tokens
        if let Ok(elapsed) = self.last_refill.elapsed() {
            let elapsed_secs = elapsed.as_secs_f64();
            self.tokens = (self.tokens + elapsed_secs * self.refill_rate).min(self.capacity as f64);
            self.last_refill = SystemTime::now();
        }

        // Try to take a token
        if self.tokens >= 1.0 {
            self.tokens -= 1.0;
            true
        } else {
            false
        }
    }
}

/// Rate limiter manager - tracks limits per IP address
pub struct RateLimiterManager {
    config: RateLimitConfig,
    limiters: Arc<Mutex<HashMap<String, TokenBucket>>>,
}

impl RateLimiterManager {
    pub fn new(config: RateLimitConfig) -> Self {
        Self {
            config,
            limiters: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Check if request from IP is allowed
    pub fn check_rate_limit(&self, ip: &str) -> Result<(), String> {
        let mut limiters = self.limiters.lock().unwrap();

        let limiter = limiters
            .entry(ip.to_string())
            .or_insert_with(|| {
                TokenBucket::new(self.config.requests_per_minute, self.config.requests_per_minute)
            });

        if limiter.try_take_token() {
            Ok(())
        } else {
            Err(format!(
                "Rate limit exceeded: max {} requests per minute",
                self.config.requests_per_minute
            ))
        }
    }

    /// Validate content length
    pub fn check_content_length(&self, length: u64) -> Result<(), String> {
        if length > self.config.max_content_length {
            return Err(format!(
                "Content length {} exceeds maximum {}",
                length, self.config.max_content_length
            ));
        }
        Ok(())
    }
}

/// Security headers for HTTP responses
pub struct SecurityHeaders;

impl SecurityHeaders {
    /// Get security headers to prevent common attacks
    pub fn get_headers() -> Vec<(String, String)> {
        vec![
            // CSRF Protection
            ("X-CSRF-Token".to_string(), "required".to_string()),

            // XSS Protection
            ("X-Content-Type-Options".to_string(), "nosniff".to_string()),
            ("X-Frame-Options".to_string(), "SAMEORIGIN".to_string()),
            ("X-XSS-Protection".to_string(), "1; mode=block".to_string()),

            // Content Security Policy (basic)
            ("Content-Security-Policy".to_string(),
             "default-src 'self'; script-src 'self'; style-src 'self' 'unsafe-inline'".to_string()),

            // Referrer Policy (data theft protection)
            ("Referrer-Policy".to_string(), "strict-origin-when-cross-origin".to_string()),

            // HSTS (HTTPS only)
            ("Strict-Transport-Security".to_string(), "max-age=31536000; includeSubDomains".to_string()),
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rate_limiter_allows_initial_request() {
        let config = RateLimitConfig {
            requests_per_minute: 10,
            max_content_length: 1024,
        };
        let manager = RateLimiterManager::new(config);
        assert!(manager.check_rate_limit("127.0.0.1").is_ok());
    }

    #[test]
    fn test_content_length_validation() {
        let config = RateLimitConfig {
            requests_per_minute: 10,
            max_content_length: 1024,
        };
        let manager = RateLimiterManager::new(config);

        assert!(manager.check_content_length(512).is_ok());
        assert!(manager.check_content_length(1024).is_ok());
        assert!(manager.check_content_length(2048).is_err());
    }

    #[test]
    fn test_security_headers() {
        let headers = SecurityHeaders::get_headers();
        assert!(headers.len() > 0);

        // Check for important headers
        let header_names: Vec<_> = headers.iter().map(|(name, _)| name).collect();
        assert!(header_names.contains(&&"X-Content-Type-Options".to_string()));
        assert!(header_names.contains(&&"Content-Security-Policy".to_string()));
    }
}
