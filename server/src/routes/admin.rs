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

#[derive(Debug, Deserialize)]
pub struct ResetPasswordRequest {
    pub new_password: String,
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

#[post("/api/admin/users/{id}/reset-password")]
pub async fn reset_user_password(
    _admin: AdminUser,
    db: web::Data<Database>,
    id: web::Path<i64>,
    body: web::Json<ResetPasswordRequest>,
) -> Result<HttpResponse, AppError> {
    let new_hash = password::hash_password(&body.new_password).map_err(AppError::BadRequest)?;
    db.update_password_hash(*id, &new_hash).await?;
    db.revoke_all_sessions_for_user(*id).await?;
    Ok(HttpResponse::Ok().json(serde_json::json!({ "status": "ok" })))
}

#[actix_web::delete("/api/admin/users/{id}")]
pub async fn delete_user(
    _admin: AdminUser,
    db: web::Data<Database>,
    id: web::Path<i64>,
) -> Result<HttpResponse, AppError> {
    let user_id = *id;
    let Some(target) = db.get_user_by_id(user_id).await? else {
        return Err(AppError::NotFound("User not found".to_string()));
    };

    if target.role.is_admin() {
        let admins = db.count_active_admins().await?;
        if admins <= 1 {
            return Err(AppError::Conflict(
                "Cannot delete the last admin account.".to_string(),
            ));
        }
    }

    db.delete_user_cascade(user_id).await?;
    Ok(HttpResponse::NoContent().finish())
}
