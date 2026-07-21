use crate::auth::hash_session_token;
use crate::db::Database;
use crate::errors::AppError;
use actix_web::body::BoxBody;
use actix_web::dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform};
use actix_web::http::header;
use actix_web::{Error, FromRequest, HttpMessage, HttpRequest, HttpResponse};
use chrono::Utc;
use futures_util::future::{ready, LocalBoxFuture, Ready};
use std::rc::Rc;

#[derive(Debug, Clone)]
pub struct McpPrincipal {
    pub user_id: i64,
    pub client_id: String,
    pub scopes: Vec<String>,
}

impl FromRequest for McpPrincipal {
    type Error = AppError;
    type Future = Ready<Result<Self, Self::Error>>;

    fn from_request(req: &HttpRequest, _payload: &mut actix_web::dev::Payload) -> Self::Future {
        let principal = req.extensions().get::<McpPrincipal>().cloned();
        ready(principal.ok_or(AppError::Unauthorized))
    }
}

#[derive(Clone)]
pub struct McpBearerAuth {
    db: Database,
}

impl McpBearerAuth {
    pub fn new(db: Database, _resource_metadata_url: String) -> Self {
        Self { db }
    }
}

impl<S, B> Transform<S, ServiceRequest> for McpBearerAuth
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: actix_web::body::MessageBody + 'static,
{
    type Response = ServiceResponse<BoxBody>;
    type Error = Error;
    type InitError = ();
    type Transform = McpBearerAuthMiddleware<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(McpBearerAuthMiddleware {
            service: Rc::new(service),
            db: self.db.clone(),
        }))
    }
}

pub struct McpBearerAuthMiddleware<S> {
    service: Rc<S>,
    db: Database,
}

impl<S, B> Service<ServiceRequest> for McpBearerAuthMiddleware<S>
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
        let auth_header = req
            .headers()
            .get(header::AUTHORIZATION)
            .and_then(|value| value.to_str().ok())
            .map(ToString::to_string);
        // Per-IP key for the pre-auth throttle. Mirrors the login limiter: prefer
        // the nginx-set X-Real-IP, fall back to the peer socket.
        let ip_key = req
            .headers()
            .get("x-real-ip")
            .and_then(|v| v.to_str().ok())
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .or_else(|| req.peer_addr().map(|a| a.ip().to_string()))
            .unwrap_or_else(|| "unknown".to_string());
        let db = self.db.clone();
        let service = self.service.clone();

        Box::pin(async move {
            // Throttle by IP BEFORE any DB work so an unauthenticated flood of bogus
            // bearer tokens cannot amplify into unbounded DB reads/writes (B-MED-3).
            if !crate::mcp::rate_limit::admit_auth_attempt(&ip_key, Utc::now()) {
                let (req, _) = req.into_parts();
                return Ok(ServiceResponse::new(req, too_many_requests_response()));
            }

            let Some(auth_header) = auth_header else {
                let (req, _) = req.into_parts();
                let resp = unauthorized_response();
                return Ok(ServiceResponse::new(req, resp));
            };

            let Some(token) = auth_header.strip_prefix("Bearer ").map(str::trim) else {
                let (req, _) = req.into_parts();
                let resp = unauthorized_response();
                return Ok(ServiceResponse::new(req, resp));
            };

            let token_hash = hash_session_token(token);
            let principal = if let Some(token_row) = db
                .get_oauth_access_token_by_hash(&token_hash)
                .await
                .map_err(Error::from)?
            {
                if token_row.revoked_at.is_some()
                    || token_row.expires_at <= Utc::now()
                    || token_row.user.must_change_password
                {
                    None
                } else {
                    Some(McpPrincipal {
                        user_id: token_row.user.id,
                        client_id: token_row.client_id,
                        scopes: token_row.scopes,
                    })
                }
            } else if let Some(token_row) = db
                .get_mcp_token_by_hash(&token_hash)
                .await
                .map_err(Error::from)?
            {
                if token_row.revoked_at.is_some()
                    || token_row
                        .expires_at
                        .is_some_and(|value| value <= Utc::now())
                    || token_row.user_disabled_at.is_some()
                    || token_row.user_must_change_password
                {
                    None
                } else {
                    // Debounce the last-used write so a valid-token flood does not
                    // force a SQLite UPDATE on every request (B-MED-3).
                    if crate::mcp::rate_limit::should_touch_token(token_row.id, Utc::now()) {
                        db.touch_mcp_token_last_used(token_row.id)
                            .await
                            .map_err(Error::from)?;
                    }
                    Some(McpPrincipal {
                        user_id: token_row.user_id,
                        client_id: format!("mcp_token:{}", token_row.id),
                        scopes: token_row.scopes,
                    })
                }
            } else {
                None
            };

            let Some(principal) = principal else {
                let (req, _) = req.into_parts();
                let resp = unauthorized_response();
                return Ok(ServiceResponse::new(req, resp));
            };

            req.extensions_mut().insert(principal);

            let res = service.call(req).await?;
            Ok(res.map_into_boxed_body())
        })
    }
}

fn too_many_requests_response() -> HttpResponse {
    HttpResponse::TooManyRequests().json(serde_json::json!({
        "error": "Too Many Requests",
        "auth_type": "bearer_token"
    }))
}

fn unauthorized_response() -> HttpResponse {
    HttpResponse::Unauthorized()
        .insert_header((
            header::WWW_AUTHENTICATE,
            r#"Bearer realm="SwoleMate MCP", error="invalid_token", error_description="Use a personal MCP token from /settings as Authorization: Bearer smcp_...""#,
        ))
        .json(serde_json::json!({
            "error": "Unauthorized",
            "auth_type": "bearer_token",
            "token_prefix": "smcp_",
            "settings_path": "/settings"
        }))
}
