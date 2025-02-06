use actix_web::{HttpResponse, ResponseError};
use log::error;
use serde_json::json;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("Database error: {0}")]
    DatabaseError(#[from] sqlx::Error),

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
            AppError::DatabaseError(e) => {
                HttpResponse::InternalServerError().json(json!({
                    "error": "Database error occurred",
                    "message": e.to_string(),
                    "error_type": "database_error"
                }))
            }
            AppError::NotFound(msg) => {
                HttpResponse::NotFound().json(json!({
                    "error": "Not found",
                    "message": msg,
                    "error_type": "not_found"
                }))
            }
            AppError::BadRequest(msg) => {
                HttpResponse::BadRequest().json(json!({
                    "error": "Bad request",
                    "message": msg,
                    "error_type": "bad_request"
                }))
            }
        }
    }
} 