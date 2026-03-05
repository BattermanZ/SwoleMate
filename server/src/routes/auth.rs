use crate::auth::password;
use crate::auth::{build_session_cookie, normalize_username, PublicUser, SessionConfig};
use crate::db::Database;
use crate::errors::AppError;
use crate::middleware::{logout_response, CurrentUser};
use actix_web::{get, post, web, HttpRequest, HttpResponse};
use chrono::{Duration as ChronoDuration, Utc};
use serde::Deserialize;
use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};

#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Deserialize)]
pub struct ChangePasswordRequest {
    pub current_password: String,
    pub new_password: String,
}

static LOGIN_FAILURES_BY_IP: OnceLock<Mutex<HashMap<String, Vec<chrono::DateTime<Utc>>>>> =
    OnceLock::new();

fn login_failures_map() -> &'static Mutex<HashMap<String, Vec<chrono::DateTime<Utc>>>> {
    LOGIN_FAILURES_BY_IP.get_or_init(|| Mutex::new(HashMap::new()))
}

fn login_rate_limit_cfg() -> (usize, ChronoDuration) {
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
    (max_attempts, ChronoDuration::seconds(window_seconds))
}

fn request_ip(req: &HttpRequest) -> Option<String> {
    req.connection_info()
        .realip_remote_addr()
        .map(|raw| {
            raw.parse::<std::net::SocketAddr>()
                .map(|addr| addr.ip().to_string())
                .unwrap_or_else(|_| raw.trim().to_string())
        })
        .filter(|s| !s.is_empty())
}

fn is_ip_rate_limited(ip: &str, now: chrono::DateTime<Utc>) -> bool {
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

fn record_ip_failure(ip: &str, now: chrono::DateTime<Utc>) {
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

fn clear_ip_failures(ip: &str) {
    let mut map = match login_failures_map().lock() {
        Ok(guard) => guard,
        Err(poisoned) => poisoned.into_inner(),
    };
    map.remove(ip);
}

#[post("/api/auth/login")]
pub async fn login(
    db: web::Data<Database>,
    req: HttpRequest,
    cfg: web::Data<SessionConfig>,
    body: web::Json<LoginRequest>,
) -> Result<HttpResponse, AppError> {
    let username = normalize_username(&body.username);
    let now = Utc::now();
    let client_ip = request_ip(&req);

    if let Some(ip) = client_ip.as_deref() {
        if is_ip_rate_limited(ip, now) {
            return Err(AppError::TooManyRequests(
                "Too many login attempts from this IP. Try again later.".to_string(),
            ));
        }
    }

    // Generic response for unknown users.
    let Some(user) = db.get_user_by_username(&username).await? else {
        if let Some(ip) = client_ip.as_deref() {
            record_ip_failure(ip, now);
        }
        tokio::time::sleep(std::time::Duration::from_millis(200)).await;
        return Err(AppError::Unauthorized);
    };

    if user.disabled_at.is_some() {
        return Err(AppError::Unauthorized);
    }

    if let Some(locked_until) = user.locked_until {
        if locked_until > now {
            return Err(AppError::TooManyRequests(
                "Too many login attempts. Try again later.".to_string(),
            ));
        }
    }

    let ok = password::verify_password(&user.password_hash, &body.password)
        .map_err(|_| AppError::Unauthorized)?;
    if !ok {
        db.record_failed_login(user.id).await?;
        if let Some(ip) = client_ip.as_deref() {
            record_ip_failure(ip, now);
        }
        tokio::time::sleep(std::time::Duration::from_millis(200)).await;
        return Err(AppError::Unauthorized);
    }

    db.reset_login_failures(user.id).await?;
    if let Some(ip) = client_ip.as_deref() {
        clear_ip_failures(ip);
    }

    let token = crate::auth::generate_session_token();
    let session_hash = crate::auth::hash_session_token(&token);
    let expires_at = now + ChronoDuration::days(cfg.session_ttl_days.max(1));

    let ua = req
        .headers()
        .get(actix_web::http::header::USER_AGENT)
        .and_then(|h| h.to_str().ok())
        .map(|s| s.to_string());
    let ip = req
        .connection_info()
        .realip_remote_addr()
        .map(|s| s.to_string());

    db.create_session(user.id, &session_hash, expires_at, None, ua, ip)
        .await?;

    let cookie = build_session_cookie(&token, &cfg);
    let mut resp = HttpResponse::Ok().json(serde_json::json!({
        "status": "ok",
        "user": PublicUser {
            id: user.id,
            username: user.username,
            role: user.role,
            must_change_password: user.must_change_password
        }
    }));
    let _ = resp.add_cookie(&cookie);
    Ok(resp)
}

#[post("/api/auth/logout")]
pub async fn logout(
    db: web::Data<Database>,
    req: HttpRequest,
    cfg: web::Data<SessionConfig>,
    _user: CurrentUser,
) -> Result<HttpResponse, AppError> {
    let token = req
        .cookie(crate::auth::SESSION_COOKIE_NAME)
        .map(|c| c.value().to_string())
        .ok_or(AppError::Unauthorized)?;
    let session_hash = crate::auth::hash_session_token(&token);
    db.revoke_session_by_hash(&session_hash).await?;
    Ok(logout_response(&cfg))
}

#[post("/api/auth/change-password")]
pub async fn change_password(
    db: web::Data<Database>,
    req: HttpRequest,
    cfg: web::Data<SessionConfig>,
    user: CurrentUser,
    body: web::Json<ChangePasswordRequest>,
) -> Result<HttpResponse, AppError> {
    let Some(user_row) = db.get_user_by_id(user.0.id).await? else {
        return Err(AppError::Unauthorized);
    };
    if user_row.disabled_at.is_some() {
        return Err(AppError::Unauthorized);
    }

    let ok = password::verify_password(&user_row.password_hash, &body.current_password)
        .map_err(|_| AppError::Unauthorized)?;
    if !ok {
        return Err(AppError::Unauthorized);
    }

    let new_hash = password::hash_password(&body.new_password).map_err(AppError::BadRequest)?;
    db.update_password_hash(user.0.id, &new_hash, false).await?;
    db.revoke_all_sessions_for_user(user.0.id).await?;

    // Create a new session immediately.
    let token = crate::auth::generate_session_token();
    let session_hash = crate::auth::hash_session_token(&token);
    let expires_at = Utc::now() + ChronoDuration::days(cfg.session_ttl_days.max(1));

    let ua = req
        .headers()
        .get(actix_web::http::header::USER_AGENT)
        .and_then(|h| h.to_str().ok())
        .map(|s| s.to_string());
    let ip = req
        .connection_info()
        .realip_remote_addr()
        .map(|s| s.to_string());

    db.create_session(user.0.id, &session_hash, expires_at, None, ua, ip)
        .await?;

    let cookie = build_session_cookie(&token, &cfg);
    let mut resp = HttpResponse::Ok().json(serde_json::json!({ "status": "ok" }));
    let _ = resp.add_cookie(&cookie);
    Ok(resp)
}

#[get("/api/auth/me")]
pub async fn me(user: CurrentUser) -> Result<HttpResponse, AppError> {
    Ok(HttpResponse::Ok().json(PublicUser::from(&user.0)))
}
