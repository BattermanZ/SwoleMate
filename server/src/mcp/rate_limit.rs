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
