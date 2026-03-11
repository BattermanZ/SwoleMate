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

pub fn is_rate_limited(key: &str, now: DateTime<Utc>) -> bool {
    let (max_requests, window) = mcp_rate_limit_cfg();
    let window_start = now - window;
    let mut map = match mcp_requests_map().lock() {
        Ok(guard) => guard,
        Err(poisoned) => poisoned.into_inner(),
    };
    let requests = map.entry(key.to_string()).or_default();
    requests.retain(|ts| *ts > window_start);
    requests.len() >= max_requests
}

pub fn record_request(key: &str, now: DateTime<Utc>) {
    let (_, window) = mcp_rate_limit_cfg();
    let window_start = now - window;
    let mut map = match mcp_requests_map().lock() {
        Ok(guard) => guard,
        Err(poisoned) => poisoned.into_inner(),
    };
    let requests = map.entry(key.to_string()).or_default();
    requests.retain(|ts| *ts > window_start);
    requests.push(now);
}
