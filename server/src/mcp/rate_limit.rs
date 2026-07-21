use chrono::{DateTime, Duration, Utc};
use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};

static MCP_REQUESTS: OnceLock<Mutex<HashMap<String, Vec<DateTime<Utc>>>>> = OnceLock::new();

fn mcp_requests_map() -> &'static Mutex<HashMap<String, Vec<DateTime<Utc>>>> {
    MCP_REQUESTS.get_or_init(|| Mutex::new(HashMap::new()))
}

fn mcp_rate_limit_cfg() -> (usize, Duration) {
    let max_requests = std::env::var("MCP_RATE_LIMIT_PER_MINUTE")
        .ok()
        .and_then(|v| v.parse::<usize>().ok())
        .unwrap_or(60)
        .max(1);
    (max_requests, Duration::minutes(1))
}

pub fn admit_request(key: &str, now: DateTime<Utc>) -> bool {
    let (max_requests, window) = mcp_rate_limit_cfg();
    let window_start = now - window;
    let mut map = match mcp_requests_map().lock() {
        Ok(guard) => guard,
        Err(poisoned) => poisoned.into_inner(),
    };

    map.retain(|_, requests| {
        requests.retain(|ts| *ts > window_start);
        !requests.is_empty()
    });

    let requests = map.entry(key.to_string()).or_default();
    if requests.len() >= max_requests {
        return false;
    }
    requests.push(now);
    true
}

static MCP_AUTH_ATTEMPTS: OnceLock<Mutex<HashMap<String, Vec<DateTime<Utc>>>>> = OnceLock::new();

fn mcp_auth_attempts_map() -> &'static Mutex<HashMap<String, Vec<DateTime<Utc>>>> {
    MCP_AUTH_ATTEMPTS.get_or_init(|| Mutex::new(HashMap::new()))
}

fn mcp_auth_rate_limit_cfg() -> (usize, Duration) {
    let max_attempts = std::env::var("MCP_AUTH_RATE_LIMIT_PER_MINUTE")
        .ok()
        .and_then(|v| v.parse::<usize>().ok())
        .unwrap_or(120)
        .max(1);
    (max_attempts, Duration::minutes(1))
}

/// Pre-auth, per-IP throttle for the MCP endpoint. Runs BEFORE the token DB
/// lookups so a flood of bogus bearer tokens cannot force two unbounded SELECTs
/// (and a write) per request against single-writer SQLite (B-MED-3). Deliberately
/// looser than the per-user limiter, which still applies after authentication.
pub fn admit_auth_attempt(ip_key: &str, now: DateTime<Utc>) -> bool {
    let (max_attempts, window) = mcp_auth_rate_limit_cfg();
    let window_start = now - window;
    let mut map = match mcp_auth_attempts_map().lock() {
        Ok(guard) => guard,
        Err(poisoned) => poisoned.into_inner(),
    };

    map.retain(|_, attempts| {
        attempts.retain(|ts| *ts > window_start);
        !attempts.is_empty()
    });

    let attempts = map.entry(ip_key.to_string()).or_default();
    if attempts.len() >= max_attempts {
        return false;
    }
    attempts.push(now);
    true
}

static MCP_TOKEN_TOUCHED: OnceLock<Mutex<HashMap<i64, DateTime<Utc>>>> = OnceLock::new();

fn mcp_token_touched_map() -> &'static Mutex<HashMap<i64, DateTime<Utc>>> {
    MCP_TOKEN_TOUCHED.get_or_init(|| Mutex::new(HashMap::new()))
}

const TOUCH_DEBOUNCE_MINUTES: i64 = 5;

/// Returns true at most once per debounce window per token, so recording a token's
/// last-used time does not force a SQLite write on every single request (B-MED-3).
pub fn should_touch_token(token_id: i64, now: DateTime<Utc>) -> bool {
    let window_start = now - Duration::minutes(TOUCH_DEBOUNCE_MINUTES);
    let mut map = match mcp_token_touched_map().lock() {
        Ok(guard) => guard,
        Err(poisoned) => poisoned.into_inner(),
    };
    map.retain(|_, ts| *ts > window_start);
    if map.get(&token_id).is_some_and(|ts| *ts > window_start) {
        return false;
    }
    map.insert(token_id, now);
    true
}

/// Clear all in-memory rate-limit state (per-user buckets, per-IP pre-auth buckets,
/// and the last-used touch debounce). Intended as a test/maintenance hook so
/// integration tests, which share one process and reuse small autoincrement user
/// and token ids across fresh databases, can start each case from a clean slate.
pub fn reset_rate_limit_state() {
    for map in [mcp_requests_map(), mcp_auth_attempts_map()] {
        let mut guard = match map.lock() {
            Ok(guard) => guard,
            Err(poisoned) => poisoned.into_inner(),
        };
        guard.clear();
    }
    let mut touched = match mcp_token_touched_map().lock() {
        Ok(guard) => guard,
        Err(poisoned) => poisoned.into_inner(),
    };
    touched.clear();
}

#[cfg(test)]
pub fn tracked_key_count() -> usize {
    let map = match mcp_requests_map().lock() {
        Ok(guard) => guard,
        Err(poisoned) => poisoned.into_inner(),
    };
    map.len()
}

#[cfg(test)]
pub fn reset() {
    let mut map = match mcp_requests_map().lock() {
        Ok(guard) => guard,
        Err(poisoned) => poisoned.into_inner(),
    };
    map.clear();
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Mutex, OnceLock};

    fn env_lock() -> &'static Mutex<()> {
        static ENV_LOCK: OnceLock<Mutex<()>> = OnceLock::new();
        ENV_LOCK.get_or_init(|| Mutex::new(()))
    }

    #[test]
    fn admit_request_is_atomic_for_the_limit_boundary() {
        let _guard = env_lock().lock().unwrap();
        reset();
        let prev = std::env::var("MCP_RATE_LIMIT_PER_MINUTE").ok();
        std::env::set_var("MCP_RATE_LIMIT_PER_MINUTE", "2");
        let now = Utc::now();
        assert!(admit_request("client:user", now));
        assert!(admit_request("client:user", now));
        assert!(!admit_request("client:user", now));
        if let Some(prev) = prev {
            std::env::set_var("MCP_RATE_LIMIT_PER_MINUTE", prev);
        } else {
            std::env::remove_var("MCP_RATE_LIMIT_PER_MINUTE");
        }
    }

    #[test]
    fn stale_keys_are_evicted_when_new_requests_arrive() {
        let _guard = env_lock().lock().unwrap();
        reset();
        let prev = std::env::var("MCP_RATE_LIMIT_PER_MINUTE").ok();
        std::env::set_var("MCP_RATE_LIMIT_PER_MINUTE", "1");
        let now = Utc::now();
        let stale = now - Duration::minutes(2);
        assert!(admit_request("same:key", stale));
        assert!(admit_request("same:key", now));
        if let Some(prev) = prev {
            std::env::set_var("MCP_RATE_LIMIT_PER_MINUTE", prev);
        } else {
            std::env::remove_var("MCP_RATE_LIMIT_PER_MINUTE");
        }
    }
}
