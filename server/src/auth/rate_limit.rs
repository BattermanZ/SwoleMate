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
    // Prefer X-Real-IP, which nginx sets from the real peer $remote_addr and
    // which a client behind the proxy cannot forge.  Fall back to the raw peer
    // socket address for direct / dev access where no proxy header is present.
    if let Some(ip) = req
        .headers()
        .get("x-real-ip")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
    {
        // Strip port if present (e.g. "1.2.3.4:1234" or "[::1]:1234").
        let ip = ip
            .parse::<std::net::SocketAddr>()
            .map(|addr| addr.ip().to_string())
            .unwrap_or(ip);
        return Some(ip);
    }
    req.peer_addr().map(|a| a.ip().to_string())
}

/// Remove every IP whose failure timestamps have all aged out of the window.
/// Called only on the write path to bound map growth without taxing reads.
fn evict_stale(map: &mut HashMap<String, Vec<DateTime<Utc>>>, window_start: DateTime<Utc>) {
    map.retain(|_, attempts| {
        attempts.retain(|ts| *ts > window_start);
        !attempts.is_empty()
    });
}

pub fn is_ip_rate_limited(ip: &str, now: DateTime<Utc>) -> bool {
    let (max_attempts, window) = login_rate_limit_cfg();
    let window_start = now - window;
    let map = match login_failures_map().lock() {
        Ok(guard) => guard,
        Err(poisoned) => poisoned.into_inner(),
    };
    // Read-only: count recent failures for this IP only. No global sweep and
    // no map insertion for unknown IPs so the check stays O(1) per call.
    map.get(ip)
        .map(|v| v.iter().filter(|ts| **ts > window_start).count())
        .unwrap_or(0)
        >= max_attempts
}

pub fn record_ip_failure(ip: &str, now: DateTime<Utc>) {
    let (_, window) = login_rate_limit_cfg();
    let window_start = now - window;
    let mut map = match login_failures_map().lock() {
        Ok(guard) => guard,
        Err(poisoned) => poisoned.into_inner(),
    };
    // Evict stale entries on the write path to bound memory growth.
    evict_stale(&mut map, window_start);
    let attempts = map.entry(ip.to_string()).or_default();
    attempts.push(now);
}

pub fn clear_ip_failures(ip: &str) {
    let mut map = match login_failures_map().lock() {
        Ok(guard) => guard,
        Err(poisoned) => poisoned.into_inner(),
    };
    map.remove(ip);
}
