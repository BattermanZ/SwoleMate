use actix_web::cookie::{time::Duration as CookieDuration, Cookie, SameSite};
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use base64::Engine;
use rand::rngs::OsRng;
use rand::RngCore;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

pub mod password;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Role {
    Admin,
    User,
}

impl Role {
    pub fn is_admin(self) -> bool {
        matches!(self, Role::Admin)
    }
}

#[derive(Debug, Clone)]
pub struct AuthUser {
    pub id: i64,
    pub username: String,
    pub role: Role,
    pub must_change_password: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct PublicUser {
    pub id: i64,
    pub username: String,
    pub role: Role,
    pub must_change_password: bool,
}

impl From<&AuthUser> for PublicUser {
    fn from(value: &AuthUser) -> Self {
        Self {
            id: value.id,
            username: value.username.clone(),
            role: value.role,
            must_change_password: value.must_change_password,
        }
    }
}

pub const SESSION_COOKIE_NAME: &str = "swolemate_session";

#[derive(Debug, Clone)]
pub struct SessionConfig {
    pub secure_cookie: bool,
    pub session_ttl_days: i64,
    pub rotate_if_expires_within_days: i64,
}

impl SessionConfig {
    pub fn for_env(app_env: &str) -> Self {
        let secure_cookie = app_env == "production";
        Self {
            secure_cookie,
            session_ttl_days: 90,
            rotate_if_expires_within_days: 30,
        }
    }
}

pub fn normalize_username(username: &str) -> String {
    username.trim().to_lowercase()
}

pub fn generate_session_token() -> String {
    let mut bytes = [0u8; 32];
    OsRng.fill_bytes(&mut bytes);
    URL_SAFE_NO_PAD.encode(bytes)
}

pub fn hash_session_token(token: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(token.as_bytes());
    let digest = hasher.finalize();
    hex::encode(digest)
}

pub fn build_session_cookie(token: &str, cfg: &SessionConfig) -> Cookie<'static> {
    Cookie::build(SESSION_COOKIE_NAME, token.to_string())
        .http_only(true)
        .same_site(SameSite::Lax)
        .secure(cfg.secure_cookie)
        .path("/")
        .max_age(CookieDuration::days(cfg.session_ttl_days))
        .finish()
}

pub fn clear_session_cookie(cfg: &SessionConfig) -> Cookie<'static> {
    Cookie::build(SESSION_COOKIE_NAME, "")
        .http_only(true)
        .same_site(SameSite::Lax)
        .secure(cfg.secure_cookie)
        .path("/")
        .max_age(CookieDuration::seconds(0))
        .finish()
}
