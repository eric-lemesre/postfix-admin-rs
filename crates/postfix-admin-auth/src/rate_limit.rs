//! Login rate limiting and brute-force protection.
//!
//! Uses an in-memory concurrent `DashMap` to track failed login attempts
//! per IP address with progressive delays and lockouts.

use std::time::{Duration, Instant};

use dashmap::DashMap;

use crate::error::AuthError;

/// Record of login attempts for a single IP address.
#[derive(Debug, Clone)]
struct AttemptRecord {
    /// Number of consecutive failed attempts.
    count: u32,
    /// Time of the first failed attempt in the current window.
    window_start: Instant,
    /// Time of the last failed attempt.
    last_attempt: Instant,
    /// Whether the IP is currently locked out.
    locked_until: Option<Instant>,
}

/// Rate limiter for login attempts, keyed by IP address.
pub struct LoginRateLimiter {
    attempts: DashMap<String, AttemptRecord>,
    max_attempts: u32,
    window: Duration,
    lockout_duration: Duration,
}

impl LoginRateLimiter {
    /// Create a new rate limiter.
    ///
    /// - `max_attempts`: maximum failed attempts before lockout
    /// - `window_secs`: time window in seconds for counting attempts
    /// - `lockout_secs`: lockout duration in seconds after max attempts
    #[must_use]
    pub fn new(max_attempts: u32, window_secs: u64, lockout_secs: u64) -> Self {
        Self {
            attempts: DashMap::new(),
            max_attempts,
            window: Duration::from_secs(window_secs),
            lockout_duration: Duration::from_secs(lockout_secs),
        }
    }

    /// Check if a login attempt is allowed for the given IP.
    ///
    /// # Errors
    ///
    /// Returns `AuthError::RateLimited` with seconds until retry if locked out.
    pub fn check_allowed(&self, ip: &str) -> Result<(), AuthError> {
        if let Some(record) = self.attempts.get(ip) {
            if let Some(locked_until) = record.locked_until {
                let now = Instant::now();
                if now < locked_until {
                    let remaining = locked_until.duration_since(now).as_secs();
                    return Err(AuthError::RateLimited(remaining));
                }
            }
        }
        Ok(())
    }

    /// Record a failed login attempt for the given IP.
    pub fn record_failure(&self, ip: &str) {
        let now = Instant::now();
        let mut entry = self
            .attempts
            .entry(ip.to_string())
            .or_insert_with(|| AttemptRecord {
                count: 0,
                window_start: now,
                last_attempt: now,
                locked_until: None,
            });

        let record = entry.value_mut();

        // Reset if window has expired
        if now.duration_since(record.window_start) > self.window {
            record.count = 0;
            record.window_start = now;
            record.locked_until = None;
        }

        record.count += 1;
        record.last_attempt = now;

        // Lock out if max attempts reached
        if record.count >= self.max_attempts {
            record.locked_until = Some(now + self.lockout_duration);
        }
    }

    /// Record a successful login, clearing the attempt counter.
    pub fn record_success(&self, ip: &str) {
        self.attempts.remove(ip);
    }

    /// Get the progressive delay for the given IP.
    ///
    /// Returns a delay of `2^(n-1)` seconds (capped at 30s) where `n` is
    /// the number of consecutive failures.
    #[must_use]
    pub fn get_delay(&self, ip: &str) -> Option<Duration> {
        self.attempts.get(ip).and_then(|record| {
            if record.count == 0 {
                return None;
            }
            let exponent = record.count.saturating_sub(1).min(4); // cap at 2^4 = 16, then 30s
            let delay_secs = 1u64.checked_shl(exponent).unwrap_or(30).min(30);
            Some(Duration::from_secs(delay_secs))
        })
    }

    /// Remove expired entries from the rate limiter.
    pub fn cleanup(&self) {
        let now = Instant::now();
        self.attempts.retain(|_, record| {
            // Keep if still within window or still locked out
            let window_active = now.duration_since(record.window_start) <= self.window;
            let lockout_active = record
                .locked_until
                .is_some_and(|locked_until| now < locked_until);
            window_active || lockout_active
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn first_attempt_is_allowed() {
        let limiter = LoginRateLimiter::new(5, 300, 900);
        assert!(limiter.check_allowed("192.168.1.1").is_ok());
    }

    #[test]
    fn blocked_after_max_attempts() {
        let limiter = LoginRateLimiter::new(3, 300, 900);
        for _ in 0..3 {
            limiter.record_failure("192.168.1.1");
        }
        let result = limiter.check_allowed("192.168.1.1");
        assert!(matches!(result, Err(AuthError::RateLimited(_))));
    }

    #[test]
    fn success_clears_failures() {
        let limiter = LoginRateLimiter::new(3, 300, 900);
        limiter.record_failure("192.168.1.1");
        limiter.record_failure("192.168.1.1");
        limiter.record_success("192.168.1.1");
        assert!(limiter.check_allowed("192.168.1.1").is_ok());
    }

    #[test]
    fn progressive_delay_increases() {
        let limiter = LoginRateLimiter::new(10, 300, 900);
        // No delay initially
        assert!(limiter.get_delay("192.168.1.1").is_none());

        limiter.record_failure("192.168.1.1"); // count=1: 2^0 = 1s
        assert_eq!(
            limiter.get_delay("192.168.1.1"),
            Some(Duration::from_secs(1))
        );

        limiter.record_failure("192.168.1.1"); // count=2: 2^1 = 2s
        assert_eq!(
            limiter.get_delay("192.168.1.1"),
            Some(Duration::from_secs(2))
        );

        limiter.record_failure("192.168.1.1"); // count=3: 2^2 = 4s
        assert_eq!(
            limiter.get_delay("192.168.1.1"),
            Some(Duration::from_secs(4))
        );
    }

    #[test]
    fn progressive_delay_capped_at_30s() {
        let limiter = LoginRateLimiter::new(20, 300, 900);
        for _ in 0..10 {
            limiter.record_failure("192.168.1.1");
        }
        let delay = limiter.get_delay("192.168.1.1");
        assert!(delay.is_some());
        assert!(delay.unwrap_or_else(|| unreachable!()) <= Duration::from_secs(30));
    }

    #[test]
    fn cleanup_removes_expired_entries() {
        let limiter = LoginRateLimiter::new(5, 0, 0); // instant expiry
        limiter.record_failure("192.168.1.1");
        std::thread::sleep(Duration::from_millis(10));
        limiter.cleanup();
        // After cleanup with 0s window, entry should be removed
        assert!(limiter.get_delay("192.168.1.1").is_none());
    }

    #[test]
    fn different_ips_are_independent() {
        let limiter = LoginRateLimiter::new(3, 300, 900);
        for _ in 0..3 {
            limiter.record_failure("192.168.1.1");
        }
        assert!(limiter.check_allowed("192.168.1.1").is_err());
        assert!(limiter.check_allowed("192.168.1.2").is_ok());
    }
}
