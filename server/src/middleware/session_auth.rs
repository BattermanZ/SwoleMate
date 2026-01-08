use crate::auth::{
    build_session_cookie, clear_session_cookie, hash_session_token, AuthUser, SessionConfig,
    SESSION_COOKIE_NAME,
};
use crate::db::Database;
use crate::errors::AppError;
use actix_web::body::BoxBody;
use actix_web::cookie::Cookie;
use actix_web::dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform};
use actix_web::{Error, FromRequest, HttpMessage, HttpRequest, HttpResponse};
use chrono::{Duration as ChronoDuration, Utc};
use futures_util::future::{ready, LocalBoxFuture, Ready};
use log::debug;
use std::rc::Rc;

#[derive(Clone)]
pub struct SessionAuth {
    db: Database,
    cfg: SessionConfig,
}

impl SessionAuth {
    pub fn new(db: Database, cfg: SessionConfig) -> Self {
        Self { db, cfg }
    }
}

#[derive(Debug, Clone)]
pub struct CurrentUser(pub AuthUser);

impl FromRequest for CurrentUser {
    type Error = AppError;
    type Future = Ready<Result<Self, Self::Error>>;

    fn from_request(req: &HttpRequest, _payload: &mut actix_web::dev::Payload) -> Self::Future {
        let user = req.extensions().get::<AuthUser>().cloned();
        ready(user.map(CurrentUser).ok_or(AppError::Unauthorized))
    }
}

#[derive(Debug, Clone)]
pub struct AdminUser(pub AuthUser);

impl FromRequest for AdminUser {
    type Error = AppError;
    type Future = Ready<Result<Self, Self::Error>>;

    fn from_request(req: &HttpRequest, _payload: &mut actix_web::dev::Payload) -> Self::Future {
        let user = req.extensions().get::<AuthUser>().cloned();
        match user {
            Some(user) if user.role.is_admin() => ready(Ok(AdminUser(user))),
            Some(_) => ready(Err(AppError::Forbidden)),
            None => ready(Err(AppError::Unauthorized)),
        }
    }
}

impl<S, B> Transform<S, ServiceRequest> for SessionAuth
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: actix_web::body::MessageBody + 'static,
{
    type Response = ServiceResponse<BoxBody>;
    type Error = Error;
    type InitError = ();
    type Transform = SessionAuthMiddleware<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(SessionAuthMiddleware {
            service: Rc::new(service),
            db: self.db.clone(),
            cfg: self.cfg.clone(),
        }))
    }
}

pub struct SessionAuthMiddleware<S> {
    service: Rc<S>,
    db: Database,
    cfg: SessionConfig,
}

fn should_skip_auth(req: &ServiceRequest) -> bool {
    if req.method() == actix_web::http::Method::OPTIONS {
        return true;
    }
    match req.path() {
        "/api/health" => true,
        "/api/auth/login" => true,
        _ => false,
    }
}

impl<S, B> Service<ServiceRequest> for SessionAuthMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: actix_web::body::MessageBody + 'static,
{
    type Response = ServiceResponse<BoxBody>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        if should_skip_auth(&req) {
            let fut = self.service.call(req);
            return Box::pin(async move { fut.await.map(ServiceResponse::map_into_boxed_body) });
        }

        let cfg = self.cfg.clone();
        let db = self.db.clone();
        let service = self.service.clone();

        // Extract cookie up-front (before moving req into async).
        let cookie_value = req
            .cookie(SESSION_COOKIE_NAME)
            .map(|c| c.value().to_string());

        Box::pin(async move {
            let Some(token) = cookie_value else {
                let (req, _) = req.into_parts();
                let resp = HttpResponse::Unauthorized().json(serde_json::json!({
                    "error": "Unauthorized"
                }));
                return Ok(ServiceResponse::new(req, resp));
            };

            let session_hash = hash_session_token(&token);
            let now = Utc::now();
            let Some((session, user)) = db
                .get_session_and_user_by_hash(&session_hash)
                .await
                .map_err(Error::from)?
            else {
                let (req, _) = req.into_parts();
                let resp = HttpResponse::Unauthorized().json(serde_json::json!({
                    "error": "Unauthorized"
                }));
                return Ok(ServiceResponse::new(req, resp));
            };

            if user.disabled_at.is_some() {
                let (req, _) = req.into_parts();
                let resp = HttpResponse::Unauthorized().json(serde_json::json!({
                    "error": "Unauthorized"
                }));
                return Ok(ServiceResponse::new(req, resp));
            }

            if session.revoked_at.is_some() || session.expires_at <= now {
                let (req, _) = req.into_parts();
                let resp = HttpResponse::Unauthorized().json(serde_json::json!({
                    "error": "Unauthorized"
                }));
                return Ok(ServiceResponse::new(req, resp));
            }

            db.touch_session(session.id).await.map_err(Error::from)?;

            let auth_user = AuthUser {
                id: user.id,
                username: user.username,
                role: user.role,
            };
            req.extensions_mut().insert(auth_user.clone());

            let rotate_cutoff =
                now + ChronoDuration::days(cfg.rotate_if_expires_within_days.max(0));
            let mut set_cookie: Option<Cookie<'static>> = None;

            if session.expires_at <= rotate_cutoff {
                let new_token = crate::auth::generate_session_token();
                let new_hash = hash_session_token(&new_token);
                let new_expires = now + ChronoDuration::days(cfg.session_ttl_days.max(1));

                let ua = req
                    .headers()
                    .get(actix_web::http::header::USER_AGENT)
                    .and_then(|h| h.to_str().ok())
                    .map(|s| s.to_string());
                let ip = req
                    .connection_info()
                    .realip_remote_addr()
                    .map(|s| s.to_string());

                db.rotate_session(
                    session.id,
                    &session.session_hash,
                    user.id,
                    &new_hash,
                    new_expires,
                    ua,
                    ip,
                )
                .await
                .map_err(Error::from)?;

                set_cookie = Some(build_session_cookie(&new_token, &cfg));
                debug!(
                    target: "auth",
                    "Rotated session for user {} (role={:?})",
                    auth_user.id,
                    auth_user.role
                );
            }

            let mut res = service.call(req).await?.map_into_boxed_body();
            if let Some(cookie) = set_cookie.take() {
                let _ = res.response_mut().add_cookie(&cookie);
            }
            Ok(res)
        })
    }
}

pub fn logout_response(cfg: &SessionConfig) -> HttpResponse {
    let cookie = clear_session_cookie(cfg);
    let mut resp = HttpResponse::Ok().json(serde_json::json!({ "status": "ok" }));
    let _ = resp.add_cookie(&cookie);
    resp
}
