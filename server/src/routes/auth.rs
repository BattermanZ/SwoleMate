use crate::auth::password;
use crate::auth::{build_session_cookie, normalize_username, PublicUser, SessionConfig};
use crate::db::Database;
use crate::errors::AppError;
use crate::middleware::{logout_response, CurrentUser};
use actix_web::{get, post, web, HttpRequest, HttpResponse};
use chrono::{Duration as ChronoDuration, Utc};
use serde::Deserialize;

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

#[post("/api/auth/login")]
pub async fn login(
    db: web::Data<Database>,
    req: HttpRequest,
    cfg: web::Data<SessionConfig>,
    body: web::Json<LoginRequest>,
) -> Result<HttpResponse, AppError> {
    let username = normalize_username(&body.username);
    let now = Utc::now();

    // Generic response for unknown users.
    let Some(user) = db.get_user_by_username(&username).await? else {
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
        tokio::time::sleep(std::time::Duration::from_millis(200)).await;
        return Err(AppError::Unauthorized);
    }

    db.reset_login_failures(user.id).await?;

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
        "user": PublicUser { id: user.id, username: user.username, role: user.role }
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
    db.update_password_hash(user.0.id, &new_hash).await?;
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
