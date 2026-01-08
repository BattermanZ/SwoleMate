use actix_web::{
    dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform},
    http::header,
    Error, HttpResponse,
};
use futures_util::future::LocalBoxFuture;
use log::info;
use std::{
    env,
    future::{ready, Ready},
    rc::Rc,
    sync::Arc,
    time::Instant,
};
use tokio::sync::Semaphore;
use uuid::Uuid;

pub struct RequestLogger;

impl<S, B> Transform<S, ServiceRequest> for RequestLogger
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type InitError = ();
    type Transform = RequestLoggerMiddleware<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(RequestLoggerMiddleware { service }))
    }
}

pub struct RequestLoggerMiddleware<S> {
    service: S,
}

impl<S, B> Service<ServiceRequest> for RequestLoggerMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let start = Instant::now();
        let request_id = Uuid::new_v4();
        let method = req.method().clone();
        let uri = req.uri().clone();

        info!(
            target: "request",
            "Request started - {{ \"request_id\": \"{}\", \"method\": \"{}\", \"path\": \"{}\" }}",
            request_id,
            method,
            uri
        );

        let fut = self.service.call(req);

        Box::pin(async move {
            let res = fut.await?;
            let duration = start.elapsed();

            info!(
                target: "request",
                "Request completed - {{ \"request_id\": \"{}\", \"method\": \"{}\", \"path\": \"{}\", \"status\": {}, \"duration_ms\": {} }}",
                request_id,
                method,
                uri,
                res.status().as_u16(),
                duration.as_millis()
            );

            Ok(res)
        })
    }
}

#[derive(Clone)]
pub struct ApiAuth {
    api_token: Option<String>,
    admin_token: Option<String>,
}

impl ApiAuth {
    pub fn new(api_token: Option<String>, admin_token: Option<String>) -> Self {
        Self {
            api_token,
            admin_token,
        }
    }

    pub fn from_env() -> Self {
        let api_token = env::var("SWOLEMATE_API_TOKEN")
            .ok()
            .filter(|s| !s.is_empty());
        let admin_token = env::var("SWOLEMATE_ADMIN_TOKEN")
            .ok()
            .filter(|s| !s.is_empty());
        Self::new(api_token, admin_token)
    }

    pub fn is_configured(&self) -> bool {
        self.api_token.is_some() || self.admin_token.is_some()
    }
}

impl<S, B> Transform<S, ServiceRequest> for ApiAuth
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: actix_web::body::MessageBody + 'static,
{
    type Response = ServiceResponse<actix_web::body::BoxBody>;
    type Error = Error;
    type InitError = ();
    type Transform = ApiAuthMiddleware<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(ApiAuthMiddleware {
            service,
            api_token: self.api_token.clone(),
            admin_token: self.admin_token.clone(),
        }))
    }
}

pub struct ApiAuthMiddleware<S> {
    service: S,
    api_token: Option<String>,
    admin_token: Option<String>,
}

fn constant_time_eq(a: &str, b: &str) -> bool {
    let a_bytes = a.as_bytes();
    let b_bytes = b.as_bytes();
    let mut diff = a_bytes.len() ^ b_bytes.len();
    let max_len = std::cmp::max(a_bytes.len(), b_bytes.len());
    for i in 0..max_len {
        let av = *a_bytes.get(i).unwrap_or(&0);
        let bv = *b_bytes.get(i).unwrap_or(&0);
        diff |= (av ^ bv) as usize;
    }
    diff == 0
}

impl<S, B> Service<ServiceRequest> for ApiAuthMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: actix_web::body::MessageBody + 'static,
{
    type Response = ServiceResponse<actix_web::body::BoxBody>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let api_token = self.api_token.clone();
        let admin_token = self.admin_token.clone();

        // Skip auth entirely if not configured (dev-mode convenience).
        // For production, main.rs can enforce tokens exist.
        if api_token.is_none() && admin_token.is_none() {
            let fut = self.service.call(req);
            return Box::pin(async move { fut.await.map(ServiceResponse::map_into_boxed_body) });
        }

        // Allow health checks and CORS preflight without auth.
        if req.path() == "/api/health" || req.method() == actix_web::http::Method::OPTIONS {
            let fut = self.service.call(req);
            return Box::pin(async move { fut.await.map(ServiceResponse::map_into_boxed_body) });
        }

        let is_admin =
            req.path().starts_with("/api/backups") || req.path().starts_with("/api/logs");
        let expected = if is_admin {
            admin_token.as_ref().or(api_token.as_ref())
        } else {
            api_token.as_ref().or(admin_token.as_ref())
        };

        let Some(expected) = expected else {
            let fut = self.service.call(req);
            return Box::pin(async move { fut.await.map(ServiceResponse::map_into_boxed_body) });
        };

        let provided = req
            .headers()
            .get(header::AUTHORIZATION)
            .and_then(|h| h.to_str().ok())
            .and_then(|raw| raw.strip_prefix("Bearer "))
            .unwrap_or("");

        if !constant_time_eq(provided, expected) {
            let (req, _) = req.into_parts();
            let resp = HttpResponse::Unauthorized().json(serde_json::json!({
                "error": "Unauthorized"
            }));
            return Box::pin(async move { Ok(ServiceResponse::new(req, resp)) });
        }

        let fut = self.service.call(req);
        Box::pin(async move { fut.await.map(ServiceResponse::map_into_boxed_body) })
    }
}

#[derive(Clone)]
pub struct ApiConcurrency {
    global: Arc<Semaphore>,
    logs: Arc<Semaphore>,
    backups: Arc<Semaphore>,
    restore: Arc<Semaphore>,
    timeout_ms: u64,
}

impl ApiConcurrency {
    pub fn from_env() -> Self {
        let global = env::var("API_MAX_INFLIGHT")
            .ok()
            .and_then(|v| v.parse::<usize>().ok())
            .unwrap_or(64);
        let logs = env::var("API_MAX_INFLIGHT_LOGS")
            .ok()
            .and_then(|v| v.parse::<usize>().ok())
            .unwrap_or(8);
        let backups = env::var("API_MAX_INFLIGHT_BACKUPS")
            .ok()
            .and_then(|v| v.parse::<usize>().ok())
            .unwrap_or(2);
        let restore = env::var("API_MAX_INFLIGHT_BACKUP_RESTORE")
            .ok()
            .and_then(|v| v.parse::<usize>().ok())
            .unwrap_or(1);
        let timeout_ms = env::var("API_CONCURRENCY_TIMEOUT_MS")
            .ok()
            .and_then(|v| v.parse::<u64>().ok())
            .unwrap_or(1500);

        Self {
            global: Arc::new(Semaphore::new(global)),
            logs: Arc::new(Semaphore::new(logs)),
            backups: Arc::new(Semaphore::new(backups)),
            restore: Arc::new(Semaphore::new(restore)),
            timeout_ms,
        }
    }

    fn classify(req: &ServiceRequest) -> &'static str {
        let path = req.path();
        if path == "/api/health" || req.method() == actix_web::http::Method::OPTIONS {
            return "skip";
        }
        if path.starts_with("/api/logs") && req.method() == actix_web::http::Method::POST {
            return "logs";
        }
        if path.starts_with("/api/backups") {
            if path.contains("/restore") && req.method() == actix_web::http::Method::POST {
                return "restore";
            }
            if matches!(
                *req.method(),
                actix_web::http::Method::POST | actix_web::http::Method::DELETE
            ) {
                return "backups";
            }
        }
        "global"
    }
}

impl<S, B> Transform<S, ServiceRequest> for ApiConcurrency
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: actix_web::body::MessageBody + 'static,
{
    type Response = ServiceResponse<actix_web::body::BoxBody>;
    type Error = Error;
    type InitError = ();
    type Transform = ApiConcurrencyMiddleware<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(ApiConcurrencyMiddleware {
            service: Rc::new(service),
            global: self.global.clone(),
            logs: self.logs.clone(),
            backups: self.backups.clone(),
            restore: self.restore.clone(),
            timeout_ms: self.timeout_ms,
        }))
    }
}

pub struct ApiConcurrencyMiddleware<S> {
    service: Rc<S>,
    global: Arc<Semaphore>,
    logs: Arc<Semaphore>,
    backups: Arc<Semaphore>,
    restore: Arc<Semaphore>,
    timeout_ms: u64,
}

impl<S, B> Service<ServiceRequest> for ApiConcurrencyMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: actix_web::body::MessageBody + 'static,
{
    type Response = ServiceResponse<actix_web::body::BoxBody>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let class = ApiConcurrency::classify(&req);
        if class == "skip" {
            let fut = self.service.call(req);
            return Box::pin(async move { fut.await.map(ServiceResponse::map_into_boxed_body) });
        }

        let service = self.service.clone();
        let global = self.global.clone();
        let logs = self.logs.clone();
        let backups = self.backups.clone();
        let restore = self.restore.clone();
        let timeout_ms = self.timeout_ms;

        Box::pin(async move {
            let acquire = async {
                let global_permit = global.clone().acquire_owned().await.map_err(|_| ())?;
                let specific_permit = match class {
                    "logs" => Some(logs.clone().acquire_owned().await.map_err(|_| ())?),
                    "backups" => Some(backups.clone().acquire_owned().await.map_err(|_| ())?),
                    "restore" => Some(restore.clone().acquire_owned().await.map_err(|_| ())?),
                    _ => None,
                };
                Ok::<_, ()>((global_permit, specific_permit))
            };

            let permits =
                tokio::time::timeout(std::time::Duration::from_millis(timeout_ms), acquire).await;

            let Ok(Ok((_global_permit, _specific_permit))) = permits else {
                let (req, _) = req.into_parts();
                let resp = HttpResponse::ServiceUnavailable().json(serde_json::json!({
                    "error": "Server busy"
                }));
                return Ok(ServiceResponse::new(req, resp));
            };

            let res = service.call(req).await?;
            Ok(res.map_into_boxed_body())
        })
    }
}
