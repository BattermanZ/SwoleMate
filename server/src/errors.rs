use actix_web::{HttpResponse, ResponseError};
use log::error;
use serde_json::json;
use thiserror::Error;

/// Parse the EXPOSE_INTERNAL_ERRORS flag truthily rather than keying on mere
/// presence, so that an operator who sets it to 0 / false to *disable* leakage
/// actually disables it (B-LOW-6). Matches the ENABLE_HSTS convention.
fn expose_flag_enabled(value: Option<&str>) -> bool {
    value
        .map(|v| v == "1" || v.eq_ignore_ascii_case("true"))
        .unwrap_or(false)
}

fn expose_internal_errors() -> bool {
    cfg!(debug_assertions)
        || expose_flag_enabled(std::env::var("EXPOSE_INTERNAL_ERRORS").ok().as_deref())
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

#[cfg(test)]
mod tests {
    use super::expose_flag_enabled;

    #[test]
    fn expose_flag_only_enabled_for_truthy_values() {
        assert!(expose_flag_enabled(Some("1")));
        assert!(expose_flag_enabled(Some("true")));
        assert!(expose_flag_enabled(Some("TRUE")));
        // Presence alone must not enable it (B-LOW-6): a disabling value stays off.
        assert!(!expose_flag_enabled(Some("0")));
        assert!(!expose_flag_enabled(Some("false")));
        assert!(!expose_flag_enabled(Some("")));
        assert!(!expose_flag_enabled(None));
    }
}
