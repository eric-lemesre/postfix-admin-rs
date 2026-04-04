//! Global API rate limiting middleware using a sliding window per IP.

use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};

use axum::extract::Request;
use axum::http::StatusCode;
use axum::middleware::Next;
use axum::response::{IntoResponse, Response};
use axum::Json;
use tokio::sync::Mutex;

use crate::error::ProblemDetails;

/// Per-IP sliding window rate limiter for the API.
#[derive(Clone)]
pub struct ApiRateLimiter {
    state: Arc<Mutex<RateLimitState>>,
    requests_per_minute: u32,
    burst_size: u32,
}

struct RateLimitState {
    windows: HashMap<String, WindowEntry>,
}

struct WindowEntry {
    count: u32,
    window_start: Instant,
}

impl ApiRateLimiter {
    /// Create a new rate limiter.
    #[must_use]
    pub fn new(requests_per_minute: u32, burst_size: u32) -> Self {
        Self {
            state: Arc::new(Mutex::new(RateLimitState {
                windows: HashMap::new(),
            })),
            requests_per_minute,
            burst_size,
        }
    }

    /// Check if a request from the given IP is allowed.
    ///
    /// Returns `(allowed, remaining, reset_secs)`.
    async fn check(&self, ip: &str) -> (bool, u32, u64) {
        let mut state = self.state.lock().await;
        let now = Instant::now();
        let window = Duration::from_secs(60);
        let limit = self.requests_per_minute + self.burst_size;

        let entry = state.windows.entry(ip.to_string()).or_insert(WindowEntry {
            count: 0,
            window_start: now,
        });

        // Reset window if expired
        if now.duration_since(entry.window_start) >= window {
            entry.count = 0;
            entry.window_start = now;
        }

        let reset_secs = window
            .saturating_sub(now.duration_since(entry.window_start))
            .as_secs();

        if entry.count >= limit {
            return (false, 0, reset_secs);
        }

        entry.count += 1;
        let remaining = limit.saturating_sub(entry.count);
        (true, remaining, reset_secs)
    }

    /// Spawn a background cleanup task that removes expired entries.
    pub fn spawn_cleanup(self: &Arc<Self>) {
        let limiter = Arc::clone(self);
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(60));
            loop {
                interval.tick().await;
                let mut state = limiter.state.lock().await;
                let now = Instant::now();
                let window = Duration::from_secs(60);
                state
                    .windows
                    .retain(|_, entry| now.duration_since(entry.window_start) < window);
            }
        });
    }
}

/// Extract client IP from request headers or connection info.
fn extract_client_ip(req: &Request) -> String {
    // Try X-Forwarded-For first
    if let Some(forwarded) = req.headers().get("x-forwarded-for") {
        if let Ok(val) = forwarded.to_str() {
            if let Some(first_ip) = val.split(',').next() {
                return first_ip.trim().to_string();
            }
        }
    }

    // Try X-Real-Ip
    if let Some(real_ip) = req.headers().get("x-real-ip") {
        if let Ok(val) = real_ip.to_str() {
            return val.trim().to_string();
        }
    }

    "unknown".to_string()
}

/// Axum middleware function for API rate limiting.
///
/// Extracts the `AppState` and checks `api_rate_limiter`. If no limiter is
/// configured, the request passes through unchanged.
pub async fn rate_limit_middleware(
    axum::extract::State(state): axum::extract::State<crate::state::AppState>,
    req: Request,
    next: Next,
) -> Response {
    let limiter = match state.api_rate_limiter {
        Some(ref l) => Arc::clone(l),
        None => return next.run(req).await,
    };

    let ip = extract_client_ip(&req);
    let (allowed, remaining, reset_secs) = limiter.check(&ip).await;
    let limit = limiter.requests_per_minute + limiter.burst_size;

    if !allowed {
        let problem = ProblemDetails {
            problem_type: "about:blank",
            title: "Too Many Requests",
            status: 429,
            detail: "API rate limit exceeded. Try again later.".to_string(),
            field: None,
        };
        let mut response = (StatusCode::TOO_MANY_REQUESTS, Json(problem)).into_response();
        let headers = response.headers_mut();
        if let Ok(val) = limit.to_string().parse() {
            headers.insert("x-ratelimit-limit", val);
        }
        if let Ok(val) = "0".parse() {
            headers.insert("x-ratelimit-remaining", val);
        }
        if let Ok(val) = reset_secs.to_string().parse() {
            headers.insert("x-ratelimit-reset", val);
        }
        return response;
    }

    let mut response = next.run(req).await;
    let headers = response.headers_mut();
    if let Ok(val) = limit.to_string().parse() {
        headers.insert("x-ratelimit-limit", val);
    }
    if let Ok(val) = remaining.to_string().parse() {
        headers.insert("x-ratelimit-remaining", val);
    }
    if let Ok(val) = reset_secs.to_string().parse() {
        headers.insert("x-ratelimit-reset", val);
    }
    response
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn allow_within_limit() {
        let limiter = ApiRateLimiter::new(10, 5);
        let (allowed, remaining, _) = limiter.check("1.2.3.4").await;
        assert!(allowed);
        assert_eq!(remaining, 14); // limit is 10 + 5 = 15, used 1
    }

    #[tokio::test]
    async fn block_over_limit() {
        let limiter = ApiRateLimiter::new(2, 0);
        let _ = limiter.check("1.2.3.4").await;
        let _ = limiter.check("1.2.3.4").await;
        let (allowed, remaining, _) = limiter.check("1.2.3.4").await;
        assert!(!allowed);
        assert_eq!(remaining, 0);
    }

    #[tokio::test]
    async fn different_ips_independent() {
        let limiter = ApiRateLimiter::new(1, 0);
        let _ = limiter.check("1.2.3.4").await;
        let (allowed, _, _) = limiter.check("5.6.7.8").await;
        assert!(allowed);
    }

    #[test]
    fn extract_ip_from_forwarded_for() {
        let req = Request::builder()
            .header("x-forwarded-for", "10.0.0.1, 10.0.0.2")
            .body(axum::body::Body::empty())
            .unwrap_or_else(|_| unreachable!());
        assert_eq!(extract_client_ip(&req), "10.0.0.1");
    }

    #[test]
    fn extract_ip_from_real_ip() {
        let req = Request::builder()
            .header("x-real-ip", "10.0.0.5")
            .body(axum::body::Body::empty())
            .unwrap_or_else(|_| unreachable!());
        assert_eq!(extract_client_ip(&req), "10.0.0.5");
    }

    #[test]
    fn extract_ip_unknown_fallback() {
        let req = Request::builder()
            .body(axum::body::Body::empty())
            .unwrap_or_else(|_| unreachable!());
        assert_eq!(extract_client_ip(&req), "unknown");
    }
}
