use actix_web::HttpRequest;
use chrono::{DateTime, Duration, Utc};
use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};

static LOGIN_FAILURES_BY_IP: OnceLock<Mutex<HashMap<String, Vec<DateTime<Utc>>>>> = OnceLock::new();

fn login_failures_map() -> &'static Mutex<HashMap<String, Vec<DateTime<Utc>>>> {
    LOGIN_FAILURES_BY_IP.get_or_init(|| Mutex::new(HashMap::new()))
}

fn login_rate_limit_cfg() -> (usize, Duration) {
    let max_attempts = std::env::var("LOGIN_RATE_LIMIT_ATTEMPTS")
        .ok()
        .and_then(|v| v.parse::<usize>().ok())
        .unwrap_or(30)
        .max(1);
    let window_seconds = std::env::var("LOGIN_RATE_LIMIT_WINDOW_SECONDS")
        .ok()
        .and_then(|v| v.parse::<i64>().ok())
        .unwrap_or(10 * 60)
        .max(30);
    (max_attempts, Duration::seconds(window_seconds))
}

pub fn request_ip(req: &HttpRequest) -> Option<String> {
    req.connection_info()
        .realip_remote_addr()
        .map(|raw| {
            raw.parse::<std::net::SocketAddr>()
                .map(|addr| addr.ip().to_string())
                .unwrap_or_else(|_| raw.trim().to_string())
        })
        .filter(|s| !s.is_empty())
}

pub fn is_ip_rate_limited(ip: &str, now: DateTime<Utc>) -> bool {
    let (max_attempts, window) = login_rate_limit_cfg();
    let window_start = now - window;
    let mut map = match login_failures_map().lock() {
        Ok(guard) => guard,
        Err(poisoned) => poisoned.into_inner(),
    };
    let attempts = map.entry(ip.to_string()).or_default();
    attempts.retain(|ts| *ts > window_start);
    attempts.len() >= max_attempts
}

pub fn record_ip_failure(ip: &str, now: DateTime<Utc>) {
    let (_, window) = login_rate_limit_cfg();
    let window_start = now - window;
    let mut map = match login_failures_map().lock() {
        Ok(guard) => guard,
        Err(poisoned) => poisoned.into_inner(),
    };
    let attempts = map.entry(ip.to_string()).or_default();
    attempts.retain(|ts| *ts > window_start);
    attempts.push(now);
}

pub fn clear_ip_failures(ip: &str) {
    let mut map = match login_failures_map().lock() {
        Ok(guard) => guard,
        Err(poisoned) => poisoned.into_inner(),
    };
    map.remove(ip);
}
