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

/// Whether we sit behind a trusted reverse proxy that sets the client-IP header.
/// Off by default: trusting `X-Real-IP` unconditionally lets a client with direct
/// network access forge its source IP to evade or poison per-IP rate limiting
/// (B-LOW-4). The Docker deployment sets `TRUST_PROXY_HEADERS=true` because the
/// server is reachable only through nginx, which sets `X-Real-IP` from the real
/// peer `$remote_addr`.
fn trust_proxy_headers() -> bool {
    std::env::var("TRUST_PROXY_HEADERS")
        .map(|v| v == "1" || v.eq_ignore_ascii_case("true"))
        .unwrap_or(false)
}

pub fn request_ip(req: &HttpRequest) -> Option<String> {
    // Only honour the proxy-supplied client IP when we're explicitly configured to
    // trust it; otherwise it is spoofable. Fall back to the raw peer socket address
    // for direct / dev access.
    if trust_proxy_headers() {
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

/// Atomically admit (or reject) a login attempt from `ip`. Under a single lock it
/// evicts stale entries and, if the IP is still under the limit, records this
/// attempt and returns `true`; otherwise it returns `false` without recording.
/// Folding the check and the increment into one critical section closes the
/// check-then-record race that let concurrent attempts overshoot the configured
/// limit (B-LOW-10), mirroring the MCP limiter's admit_request design. A
/// subsequent successful login clears the IP via `clear_ip_failures`.
pub fn admit_login_attempt(ip: &str, now: DateTime<Utc>) -> bool {
    let (max_attempts, window) = login_rate_limit_cfg();
    let window_start = now - window;
    let mut map = match login_failures_map().lock() {
        Ok(guard) => guard,
        Err(poisoned) => poisoned.into_inner(),
    };
    // Evict stale entries on the write path to bound memory growth.
    evict_stale(&mut map, window_start);
    let attempts = map.entry(ip.to_string()).or_default();
    let recent = attempts.iter().filter(|ts| **ts > window_start).count();
    if recent >= max_attempts {
        return false;
    }
    attempts.push(now);
    true
}

pub fn clear_ip_failures(ip: &str) {
    let mut map = match login_failures_map().lock() {
        Ok(guard) => guard,
        Err(poisoned) => poisoned.into_inner(),
    };
    map.remove(ip);
}
