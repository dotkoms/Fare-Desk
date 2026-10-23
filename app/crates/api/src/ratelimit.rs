use std::collections::HashMap;
use std::sync::Mutex;
use std::time::{Duration, Instant};

/// A small fixed-window limiter. One window per client; the window resets a
/// minute after the first request. Enough to keep one client from spending the
/// fare-source quota.
#[derive(Debug)]
pub struct RateLimiter {
    limit: u32,
    window: Duration,
    buckets: Mutex<HashMap<String, Bucket>>,
}

#[derive(Debug, Clone, Copy)]
struct Bucket {
    started: Instant,
    used: u32,
}

impl RateLimiter {
    pub fn new(limit_per_minute: u32) -> Self {
        Self {
            limit: limit_per_minute,
            window: Duration::from_secs(60),
            buckets: Mutex::new(HashMap::new()),
        }
    }

    /// `Ok(())` when the request may proceed; `Err(retry_after_seconds)` when
    /// the client has spent its window.
    pub fn check(&self, key: &str, now: Instant) -> Result<(), u64> {
        if self.limit == 0 {
            return Ok(());
        }
        let mut buckets = self.buckets.lock().expect("rate limiter lock");

        // Keep the map bounded under a spray of distinct keys.
        if buckets.len() > 10_000 {
            buckets.retain(|_, bucket| now.duration_since(bucket.started) < self.window);
        }

        let bucket = buckets.entry(key.to_string()).or_insert(Bucket {
            started: now,
            used: 0,
        });

        if now.duration_since(bucket.started) >= self.window {
            bucket.started = now;
            bucket.used = 0;
        }

        if bucket.used >= self.limit {
            let elapsed = now.duration_since(bucket.started).as_secs();
            return Err(self.window.as_secs().saturating_sub(elapsed).max(1));
        }

        bucket.used += 1;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn allows_up_to_the_limit_then_refuses() {
        let limiter = RateLimiter::new(2);
        let now = Instant::now();
        assert!(limiter.check("a", now).is_ok());
        assert!(limiter.check("a", now).is_ok());
        assert_eq!(limiter.check("a", now), Err(60));
    }

    #[test]
    fn windows_reset() {
        let limiter = RateLimiter::new(1);
        let now = Instant::now();
        assert!(limiter.check("a", now).is_ok());
        assert!(limiter.check("a", now).is_err());
        assert!(limiter.check("a", now + Duration::from_secs(61)).is_ok());
    }

    #[test]
    fn clients_are_independent() {
        let limiter = RateLimiter::new(1);
        let now = Instant::now();
        assert!(limiter.check("a", now).is_ok());
        assert!(limiter.check("b", now).is_ok());
    }

    #[test]
    fn zero_disables_the_limiter() {
        let limiter = RateLimiter::new(0);
        let now = Instant::now();
        for _ in 0..100 {
            assert!(limiter.check("a", now).is_ok());
        }
    }
}
