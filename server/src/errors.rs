use actix_web::{HttpResponse, ResponseError};
use log::error;
use serde_json::json;
use thiserror::Error;

fn expose_internal_errors() -> bool {
    cfg!(debug_assertions) || std::env::var("EXPOSE_INTERNAL_ERRORS").is_ok()
}

#[derive(Error, Debug)]
pub enum AppError {
    #[error("Database error: {0}")]
    DatabaseError(#[from] sqlx::Error),

    #[error("Internal error: {0}")]
    InternalError(String),

    #[error("Unauthorized")]
    Unauthorized,

    #[error("Forbidden")]
    Forbidden,

    #[error("Too many requests: {0}")]
    TooManyRequests(String),

    #[error("Conflict: {0}")]
    Conflict(String),

    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Bad request: {0}")]
    BadRequest(String),
}

impl ResponseError for AppError {
    fn error_response(&self) -> HttpResponse {
        // Log the error with structured context
        let error_type = match self {
            AppError::DatabaseError(_) => "database_error",
            AppError::InternalError(_) => "internal_error",
            AppError::Unauthorized => "unauthorized",
            AppError::Forbidden => "forbidden",
            AppError::TooManyRequests(_) => "too_many_requests",
            AppError::Conflict(_) => "conflict",
            AppError::NotFound(_) => "not_found",
            AppError::BadRequest(_) => "bad_request",
        };

        error!(
            target: "error",
            "{}",
            json!({
                "event": "error_occurred",
                "error_type": error_type,
                "error_message": self.to_string(),
                "error_details": format!("{:?}", self)
            })
        );

        match self {
            AppError::DatabaseError(e) => HttpResponse::InternalServerError().json(json!({
                "error": if expose_internal_errors() {
                    format!("Database error: {}", e)
                } else {
                    "Database error".to_string()
                }
            })),
            AppError::InternalError(msg) => HttpResponse::InternalServerError().json(json!({
                "error": if expose_internal_errors() {
                    msg.clone()
                } else {
                    "Internal server error".to_string()
                }
            })),
            AppError::Unauthorized => HttpResponse::Unauthorized().json(json!({
                "error": "Unauthorized"
            })),
            AppError::Forbidden => HttpResponse::Forbidden().json(json!({
                "error": "Forbidden"
            })),
            AppError::TooManyRequests(msg) => HttpResponse::TooManyRequests().json(json!({
                "error": msg
            })),
            AppError::Conflict(msg) => HttpResponse::Conflict().json(json!({
                "error": msg
            })),
            AppError::BadRequest(msg) => HttpResponse::BadRequest().json(json!({
                "error": msg
            })),
            AppError::NotFound(msg) => HttpResponse::NotFound().json(json!({
                "error": msg
            })),
        }
    }
}
