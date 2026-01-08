use crate::auth::{normalize_username, Role};
use crate::auth::password;
use crate::db::Database;
use crate::errors::AppError;
use crate::middleware::AdminUser;
use actix_web::{get, post, web, HttpResponse};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub struct CreateUserRequest {
    pub username: String,
    pub password: String,
    #[serde(default)]
    pub role: Option<Role>,
}

#[derive(Debug, Serialize)]
pub struct UserListItem {
    pub id: i64,
    pub username: String,
    pub role: Role,
    pub disabled_at: Option<DateTime<Utc>>,
}

#[get("/api/admin/users")]
pub async fn list_users(_admin: AdminUser, db: web::Data<Database>) -> Result<HttpResponse, AppError> {
    let users = db.list_users().await?;
    Ok(HttpResponse::Ok().json(
        users
            .into_iter()
            .map(|(id, username, role, disabled_at)| UserListItem {
                id,
                username,
                role,
                disabled_at,
            })
            .collect::<Vec<_>>(),
    ))
}

#[post("/api/admin/users")]
pub async fn create_user(
    _admin: AdminUser,
    db: web::Data<Database>,
    body: web::Json<CreateUserRequest>,
) -> Result<HttpResponse, AppError> {
    let username = normalize_username(&body.username);
    if username.is_empty() || username.len() > 64 {
        return Err(AppError::BadRequest(
            "username must be between 1 and 64 characters".to_string(),
        ));
    }
    let password_hash = password::hash_password(&body.password).map_err(AppError::BadRequest)?;
    let role = body.role.unwrap_or(Role::User);
    let id = db.create_user(&username, &password_hash, role).await?;
    Ok(HttpResponse::Created().json(serde_json::json!({ "id": id })))
}

#[post("/api/admin/users/{id}/disable")]
pub async fn disable_user(
    _admin: AdminUser,
    db: web::Data<Database>,
    id: web::Path<i64>,
) -> Result<HttpResponse, AppError> {
    db.disable_user(*id).await?;
    db.revoke_all_sessions_for_user(*id).await?;
    Ok(HttpResponse::Ok().json(serde_json::json!({ "status": "ok" })))
}
