use actix_web::{
    dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform},
    Error,
};
use futures_util::future::LocalBoxFuture;
use log::info;
use std::{
    future::{ready, Ready},
    time::Instant,
};
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
