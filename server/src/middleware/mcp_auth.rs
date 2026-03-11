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
    resource_metadata_url: String,
}

impl McpBearerAuth {
    pub fn new(db: Database, resource_metadata_url: String) -> Self {
        Self {
            db,
            resource_metadata_url,
        }
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
            resource_metadata_url: self.resource_metadata_url.clone(),
        }))
    }
}

pub struct McpBearerAuthMiddleware<S> {
    service: Rc<S>,
    db: Database,
    resource_metadata_url: String,
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
        let db = self.db.clone();
        let service = self.service.clone();
        let resource_metadata_url = self.resource_metadata_url.clone();

        Box::pin(async move {
            let Some(auth_header) = auth_header else {
                let (req, _) = req.into_parts();
                let resp = unauthorized_response(&resource_metadata_url);
                return Ok(ServiceResponse::new(req, resp));
            };

            let Some(token) = auth_header.strip_prefix("Bearer ").map(str::trim) else {
                let (req, _) = req.into_parts();
                let resp = unauthorized_response(&resource_metadata_url);
                return Ok(ServiceResponse::new(req, resp));
            };

            let token_hash = hash_session_token(token);
            let Some(token_row) = db
                .get_oauth_access_token_by_hash(&token_hash)
                .await
                .map_err(Error::from)?
            else {
                let (req, _) = req.into_parts();
                let resp = unauthorized_response(&resource_metadata_url);
                return Ok(ServiceResponse::new(req, resp));
            };

            if token_row.revoked_at.is_some()
                || token_row.expires_at <= Utc::now()
                || token_row.user.must_change_password
            {
                let (req, _) = req.into_parts();
                let resp = unauthorized_response(&resource_metadata_url);
                return Ok(ServiceResponse::new(req, resp));
            }

            req.extensions_mut().insert(McpPrincipal {
                user_id: token_row.user.id,
                client_id: token_row.client_id,
                scopes: token_row.scopes,
            });

            let res = service.call(req).await?;
            Ok(res.map_into_boxed_body())
        })
    }
}

fn unauthorized_response(resource_metadata_url: &str) -> HttpResponse {
    HttpResponse::Unauthorized()
        .insert_header((
            header::WWW_AUTHENTICATE,
            format!(
                r#"Bearer resource_metadata="{}", error="invalid_token""#,
                resource_metadata_url
            ),
        ))
        .json(serde_json::json!({
            "error": "Unauthorized"
        }))
}
