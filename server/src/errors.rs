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

    #[error("Internal server error: {0}")]
    InternalError(String),
}

impl ResponseError for AppError {
    fn error_response(&self) -> HttpResponse {
        // Log the error with context
        error!("Error occurred: {:?}", self);

        match self {
            AppError::DatabaseError(e) => {
                HttpResponse::InternalServerError().json(json!({
                    "error": "Database error occurred",
                    "message": e.to_string()
                }))
            }
            AppError::NotFound(msg) => {
                HttpResponse::NotFound().json(json!({
                    "error": "Not found",
                    "message": msg
                }))
            }
            AppError::BadRequest(msg) => {
                HttpResponse::BadRequest().json(json!({
                    "error": "Bad request",
                    "message": msg
                }))
            }
            AppError::InternalError(msg) => {
                HttpResponse::InternalServerError().json(json!({
                    "error": "Internal server error",
                    "message": msg
                }))
            }
        }
    }
} 