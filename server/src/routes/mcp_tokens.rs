use crate::auth::{generate_session_token, hash_session_token};
use crate::db::Database;
use crate::errors::AppError;
use crate::middleware::CurrentUser;
use crate::services::authz::{normalize_scopes, McpScope};
use actix_web::{get, post, web, HttpResponse};
use chrono::{Duration, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub struct CreateMcpTokenRequest {
    pub name: String,
    pub scopes: Vec<String>,
    #[serde(default)]
    pub expires_in_days: Option<i64>,
}

#[derive(Debug, Serialize)]
struct McpTokenResponse {
    id: i64,
    name: String,
    scopes: Vec<String>,
    expires_at: Option<chrono::DateTime<Utc>>,
    revoked_at: Option<chrono::DateTime<Utc>>,
    last_used_at: Option<chrono::DateTime<Utc>>,
    created_at: chrono::DateTime<Utc>,
}

#[derive(Debug, Serialize)]
struct CreatedMcpTokenResponse {
    id: i64,
    token: String,
    name: String,
    scopes: Vec<String>,
    expires_at: Option<chrono::DateTime<Utc>>,
}

const ALLOWED_SCOPES: &[&str] = &[
    McpScope::WorkoutsRead.as_str(),
    McpScope::ProgressRead.as_str(),
    McpScope::WorkoutsWrite.as_str(),
];
const DEFAULT_TOKEN_EXPIRY_DAYS: i64 = 30;

fn validate_name(name: &str) -> Result<String, AppError> {
    let trimmed = name.trim();
    if trimmed.is_empty() {
        return Err(AppError::BadRequest("name is required".to_string()));
    }
    if trimmed.chars().count() > 100 {
        return Err(AppError::BadRequest(
            "name must be 100 characters or fewer".to_string(),
        ));
    }
    Ok(trimmed.to_string())
}

fn validate_scopes(scopes: &[String]) -> Result<Vec<String>, AppError> {
    if scopes.is_empty() {
        return Err(AppError::BadRequest(
            "at least one scope is required".to_string(),
        ));
    }

    let normalized = normalize_scopes(&scopes.join(" "));
    if normalized.is_empty() {
        return Err(AppError::BadRequest(
            "at least one valid scope is required".to_string(),
        ));
    }
    if normalized
        .iter()
        .any(|scope| !ALLOWED_SCOPES.iter().any(|allowed| allowed == scope))
    {
        return Err(AppError::BadRequest(
            "unsupported scope requested".to_string(),
        ));
    }
    Ok(normalized)
}

fn validate_expiry(days: Option<i64>) -> Result<Option<chrono::DateTime<Utc>>, AppError> {
    match days {
        None => Ok(Some(Utc::now() + Duration::days(DEFAULT_TOKEN_EXPIRY_DAYS))),
        Some(days) if (1..=365).contains(&days) => Ok(Some(Utc::now() + Duration::days(days))),
        Some(_) => Err(AppError::BadRequest(
            "expires_in_days must be between 1 and 365".to_string(),
        )),
    }
}

fn map_token_row(row: crate::db::mcp_tokens::McpTokenRow) -> McpTokenResponse {
    McpTokenResponse {
        id: row.id,
        name: row.name,
        scopes: row.scopes,
        expires_at: row.expires_at,
        revoked_at: row.revoked_at,
        last_used_at: row.last_used_at,
        created_at: row.created_at,
    }
}

fn token_creation_json(body: CreatedMcpTokenResponse) -> HttpResponse {
    HttpResponse::Created()
        .insert_header((actix_web::http::header::CACHE_CONTROL, "no-store"))
        .insert_header((actix_web::http::header::PRAGMA, "no-cache"))
        .json(body)
}

#[get("/api/mcp/tokens")]
pub async fn list_mcp_tokens(
    user: CurrentUser,
    db: web::Data<Database>,
) -> Result<HttpResponse, AppError> {
    let tokens = db.list_mcp_tokens_for_user(user.0.id).await?;
    Ok(HttpResponse::Ok().json(tokens.into_iter().map(map_token_row).collect::<Vec<_>>()))
}

#[post("/api/mcp/tokens")]
pub async fn create_mcp_token(
    user: CurrentUser,
    db: web::Data<Database>,
    body: web::Json<CreateMcpTokenRequest>,
) -> Result<HttpResponse, AppError> {
    let name = validate_name(&body.name)?;
    let scopes = validate_scopes(&body.scopes)?;
    let expires_at = validate_expiry(body.expires_in_days)?;

    let raw_secret = generate_session_token();
    let raw_token = format!("smcp_{raw_secret}");
    let token_hash = hash_session_token(&raw_token);
    let scopes_json = serde_json::to_string(&scopes)
        .map_err(|e| AppError::InternalError(format!("failed to encode scopes: {e}")))?;

    let id = db
        .create_mcp_token(user.0.id, &name, &token_hash, &scopes_json, expires_at)
        .await?;

    Ok(token_creation_json(CreatedMcpTokenResponse {
        id,
        token: raw_token,
        name,
        scopes,
        expires_at,
    }))
}

#[post("/api/mcp/tokens/{id}/revoke")]
pub async fn revoke_mcp_token(
    user: CurrentUser,
    db: web::Data<Database>,
    id: web::Path<i64>,
) -> Result<HttpResponse, AppError> {
    let revoked = db.revoke_mcp_token_for_user(*id, user.0.id).await?;
    if !revoked {
        return Err(AppError::NotFound("MCP token not found".to_string()));
    }
    Ok(HttpResponse::Ok().json(serde_json::json!({
        "message": "MCP token revoked successfully"
    })))
}

#[post("/api/mcp/tokens/{id}/rotate")]
pub async fn rotate_mcp_token(
    user: CurrentUser,
    db: web::Data<Database>,
    id: web::Path<i64>,
) -> Result<HttpResponse, AppError> {
    let raw_secret = generate_session_token();
    let raw_token = format!("smcp_{raw_secret}");
    let token_hash = hash_session_token(&raw_token);

    let Some((new_id, name, scopes, expires_at)) = db
        .rotate_mcp_token_for_user(*id, user.0.id, &token_hash)
        .await?
    else {
        return Err(AppError::NotFound("MCP token not found".to_string()));
    };

    Ok(token_creation_json(CreatedMcpTokenResponse {
        id: new_id,
        token: raw_token,
        name,
        scopes,
        expires_at,
    }))
}
