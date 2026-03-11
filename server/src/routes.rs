use crate::backup::{self, BackupType};
use crate::middleware::{AdminUser, CurrentUser};
use crate::services::{exercises, progress, workouts};
use crate::{db::Database, errors::AppError, models::*};
use actix_web::{delete, get, post, put, web, HttpResponse};
use log::{debug, error, info, warn};
use serde::Deserialize;
use serde_json::json;

pub mod admin;
pub mod auth;

#[derive(Debug, Deserialize)]
pub struct ExerciseTypeQuery {
    pub exercise_type: String,
}

#[get("/api/health")]
pub async fn health_check() -> HttpResponse {
    HttpResponse::Ok().json(json!({
        "status": "healthy",
        "timestamp": chrono::Utc::now()
    }))
}

#[post("/api/workouts")]
pub async fn create_workout(
    user: CurrentUser,
    db: web::Data<Database>,
    workout_req: web::Json<CreateWorkoutRequest>,
) -> Result<HttpResponse, AppError> {
    workout_req
        .validate()
        .map_err(|e| AppError::BadRequest(e))?;
    let workout_id = workouts::create_workout(db.get_ref(), user.0.id, &workout_req.0).await?;
    Ok(HttpResponse::Created().json(json!({
        "id": workout_id,
        "message": "Workout created successfully"
    })))
}

#[put("/api/workouts/{id}/end")]
pub async fn end_workout(
    user: CurrentUser,
    db: web::Data<Database>,
    id: web::Path<i64>,
    end_req: web::Json<UpdateWorkoutRequest>,
) -> Result<HttpResponse, AppError> {
    end_req.validate().map_err(|e| AppError::BadRequest(e))?;
    workouts::end_workout(db.get_ref(), user.0.id, *id, &end_req.0).await?;
    Ok(HttpResponse::Ok().json(json!({
        "message": "Workout ended successfully"
    })))
}

#[put("/api/workouts/{id}/times")]
pub async fn update_workout_times(
    user: CurrentUser,
    db: web::Data<Database>,
    id: web::Path<i64>,
    req: web::Json<UpdateWorkoutTimesRequest>,
) -> Result<HttpResponse, AppError> {
    req.validate().map_err(|e| AppError::BadRequest(e))?;

    workouts::update_workout_times(db.get_ref(), user.0.id, *id, &req.0).await?;

    Ok(HttpResponse::Ok().json(json!({
        "message": "Workout times updated successfully"
    })))
}

#[get("/api/workouts/{id}")]
pub async fn get_workout(
    user: CurrentUser,
    db: web::Data<Database>,
    id: web::Path<i64>,
) -> Result<HttpResponse, AppError> {
    let detail = workouts::get_workout_detail(db.get_ref(), user.0.id, *id).await?;
    Ok(HttpResponse::Ok().json(detail))
}

#[get("/api/workouts")]
pub async fn get_workouts(
    user: CurrentUser,
    db: web::Data<Database>,
) -> Result<HttpResponse, AppError> {
    let workouts = workouts::list_workouts(db.get_ref(), user.0.id).await?;
    Ok(HttpResponse::Ok().json(workouts))
}

#[post("/api/workouts/{workout_id}/exercises")]
pub async fn create_exercise(
    user: CurrentUser,
    db: web::Data<Database>,
    workout_id: web::Path<i64>,
    exercise_req: web::Json<CreateExerciseRequest>,
) -> Result<HttpResponse, AppError> {
    exercise_req
        .validate()
        .map_err(|e| AppError::BadRequest(e))?;
    let exercise_id =
        exercises::create_exercise(db.get_ref(), user.0.id, *workout_id, &exercise_req.0).await?;
    Ok(HttpResponse::Created().json(json!({
        "id": exercise_id,
        "message": "Exercise created successfully"
    })))
}

#[put("/api/exercises/{id}/end")]
pub async fn end_exercise(
    user: CurrentUser,
    db: web::Data<Database>,
    id: web::Path<i64>,
    end_req: web::Json<UpdateExerciseRequest>,
) -> Result<HttpResponse, AppError> {
    end_req.validate().map_err(|e| AppError::BadRequest(e))?;
    exercises::end_exercise(db.get_ref(), user.0.id, *id, &end_req.0).await?;
    Ok(HttpResponse::Ok().json(json!({
        "message": "Exercise ended successfully"
    })))
}

#[post("/api/exercises/{exercise_id}/sets")]
pub async fn create_set(
    user: CurrentUser,
    db: web::Data<Database>,
    exercise_id: web::Path<i64>,
    set_req: web::Json<CreateSetRequest>,
) -> Result<HttpResponse, AppError> {
    set_req.validate().map_err(|e| AppError::BadRequest(e))?;
    let set_id = exercises::create_set(db.get_ref(), user.0.id, *exercise_id, &set_req.0).await?;
    Ok(HttpResponse::Created().json(json!({
        "id": set_id,
        "message": "Set created successfully"
    })))
}

#[put("/api/exercises/{exercise_id}/sets")]
pub async fn replace_sets(
    user: CurrentUser,
    db: web::Data<Database>,
    exercise_id: web::Path<i64>,
    sets_req: web::Json<Vec<CreateSetRequest>>,
) -> Result<HttpResponse, AppError> {
    for s in sets_req.iter() {
        s.validate().map_err(|e| AppError::BadRequest(e))?;
    }
    let sets = exercises::replace_sets(db.get_ref(), user.0.id, *exercise_id, &sets_req.0).await?;
    Ok(HttpResponse::Ok().json(sets))
}

#[derive(Debug, Deserialize)]
struct ClientLogEntry {
    #[serde(default)]
    timestamp: Option<String>,
    #[serde(default)]
    level: Option<String>,
    #[serde(default)]
    target: Option<String>,
    #[serde(default)]
    message: Option<String>,
    #[serde(default)]
    msg: Option<String>,
    #[serde(default)]
    metadata: Option<serde_json::Value>,
}

fn sanitize_log_field(value: &str, max_len: usize) -> String {
    let mut out = String::with_capacity(value.len().min(max_len));
    for c in value.chars() {
        if c.is_control() {
            out.push(' ');
        } else {
            out.push(c);
        }
        if out.len() >= max_len {
            break;
        }
    }
    out.trim().to_string()
}

#[post("/api/logs")]
pub async fn write_logs(_user: CurrentUser, logs: web::Json<Vec<ClientLogEntry>>) -> HttpResponse {
    const MAX_LOG_ENTRIES: usize = 1000;
    if logs.len() > MAX_LOG_ENTRIES {
        return HttpResponse::PayloadTooLarge().json(json!({
            "error": "Too many log entries"
        }));
    }

    for entry in logs.iter() {
        let level = entry
            .level
            .as_deref()
            .unwrap_or("info")
            .trim()
            .to_ascii_lowercase();
        let target = sanitize_log_field(entry.target.as_deref().unwrap_or("app"), 64);
        let target = if target.is_empty() {
            "app".to_string()
        } else {
            target
        };
        let message = entry
            .message
            .as_deref()
            .or(entry.msg.as_deref())
            .unwrap_or("<missing message>")
            .to_string();
        let message = sanitize_log_field(&message, 2000);

        let mut prefix = format!("component={}", target);
        if let Some(ts) = entry.timestamp.as_deref() {
            if !ts.is_empty() {
                prefix.push_str(&format!(" client_ts={}", ts));
            }
        }

        match level.as_str() {
            "debug" => {
                let metadata_str = entry.metadata.as_ref().and_then(|value| {
                    let s = serde_json::to_string(value).ok()?;
                    if s.len() <= 2048 {
                        Some(s)
                    } else {
                        Some(format!("{}…", &s[..2048]))
                    }
                });
                if let Some(meta) = metadata_str {
                    debug!(target: "client", "{} {} | {}", prefix, message, meta);
                    continue;
                }
                debug!(target: "client", "{} {}", prefix, message);
            }
            "warn" | "warning" => {
                warn!(target: "client", "{} {}", prefix, message);
            }
            "error" => {
                error!(target: "client", "{} {}", prefix, message);
            }
            _ => {
                info!(target: "client", "{} {}", prefix, message);
            }
        }
    }

    HttpResponse::Ok().json(json!({ "status": "ok" }))
}

#[get("/api/exercises/types")]
pub async fn get_exercise_types(
    user: CurrentUser,
    db: web::Data<Database>,
) -> Result<HttpResponse, AppError> {
    let types = exercises::list_exercise_types(db.get_ref(), user.0.id).await?;
    Ok(HttpResponse::Ok().json(types))
}

#[get("/api/exercises/last/{exercise_type}")]
pub async fn get_last_exercise_data(
    user: CurrentUser,
    db: web::Data<Database>,
    exercise_type: web::Path<String>,
) -> Result<HttpResponse, AppError> {
    let decoded_type = urlencoding::decode(&exercise_type)
        .map_err(|e| AppError::BadRequest(format!("Invalid exercise type: {}", e)))?
        .into_owned();
    let data = exercises::get_last_exercise_data(db.get_ref(), user.0.id, &decoded_type).await?;
    Ok(HttpResponse::Ok().json(data))
}

#[delete("/api/exercises/{id}")]
pub async fn cancel_exercise(
    user: CurrentUser,
    db: web::Data<Database>,
    id: web::Path<i64>,
) -> Result<HttpResponse, AppError> {
    exercises::delete_exercise(db.get_ref(), user.0.id, *id).await?;
    Ok(HttpResponse::Ok().json(json!({
        "message": "Exercise canceled successfully"
    })))
}

#[delete("/api/workouts/{id}")]
pub async fn cancel_workout(
    user: CurrentUser,
    db: web::Data<Database>,
    id: web::Path<i64>,
) -> Result<HttpResponse, AppError> {
    workouts::delete_workout(db.get_ref(), user.0.id, *id).await?;
    Ok(HttpResponse::Ok().json(json!({
        "message": "Workout canceled successfully"
    })))
}

#[get("/api/backups")]
pub async fn list_backups(_admin: AdminUser) -> Result<HttpResponse, AppError> {
    let backups = backup::list_backups().await.map_err(|e| {
        error!("Failed to list backups: {}", e);
        AppError::InternalError(e.to_string())
    })?;
    Ok(HttpResponse::Ok().json(backups))
}

#[post("/api/backups")]
pub async fn create_manual_backup(_admin: AdminUser) -> Result<HttpResponse, AppError> {
    let backup_info = backup::create_backup(BackupType::Manual)
        .await
        .map_err(|e| {
            error!("Failed to create backup: {}", e);
            AppError::InternalError(e.to_string())
        })?;
    Ok(HttpResponse::Created().json(backup_info))
}

#[post("/api/backups/{filename}/restore")]
pub async fn restore_backup(
    _admin: AdminUser,
    filename: web::Path<String>,
    db: web::Data<Database>,
) -> Result<HttpResponse, AppError> {
    if !is_safe_backup_filename(&filename) {
        return Err(AppError::BadRequest("Invalid backup filename".to_string()));
    }

    // Close all existing connections in the pool
    let current_pool = db.pool().await;
    current_pool.close().await;

    // Wait for all connections to be dropped
    tokio::time::sleep(std::time::Duration::from_millis(500)).await;

    // Perform the restore
    backup::restore_backup(&filename).await.map_err(|e| {
        error!("Failed to restore backup: {}", e);
        AppError::InternalError(e.to_string())
    })?;

    // Wait for filesystem operations to complete
    tokio::time::sleep(std::time::Duration::from_millis(100)).await;

    // Create a new connection pool with WAL mode disabled temporarily
    let db_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "sqlite:database/swolemate.db".to_string());

    // Try to connect multiple times with increasing delays
    let mut retry_count = 0;
    let max_retries = 3;
    let mut last_error = None;

    while retry_count < max_retries {
        match sqlx::sqlite::SqlitePoolOptions::new()
            .max_connections(5)
            .connect(&db_url)
            .await
        {
            Ok(new_pool) => {
                // Disable WAL mode temporarily to ensure database consistency
                if let Err(e) = sqlx::query("PRAGMA journal_mode = DELETE")
                    .execute(&new_pool)
                    .await
                {
                    error!("Failed to disable WAL mode: {}", e);
                    new_pool.close().await;
                    retry_count += 1;
                    tokio::time::sleep(std::time::Duration::from_millis(
                        100 * (retry_count as u64),
                    ))
                    .await;
                    continue;
                }

                // Re-enable WAL mode and other optimizations
                let mut pragma_ok = true;
                for pragma in [
                    "PRAGMA journal_mode = WAL",
                    "PRAGMA synchronous = NORMAL",
                    "PRAGMA foreign_keys = ON",
                    "PRAGMA busy_timeout = 5000",
                ] {
                    if let Err(e) = sqlx::query(pragma).execute(&new_pool).await {
                        error!("Failed to set pragma {}: {}", pragma, e);
                        last_error = Some(e);
                        pragma_ok = false;
                        break;
                    }
                }

                if !pragma_ok {
                    new_pool.close().await;
                    retry_count += 1;
                    tokio::time::sleep(std::time::Duration::from_millis(
                        100 * (retry_count as u64),
                    ))
                    .await;
                    continue;
                }

                // Update the database instance with the new pool
                db.replace_pool(new_pool).await;
                return Ok(HttpResponse::Ok().json(json!({
                    "message": "Backup restored successfully"
                })));
            }
            Err(e) => {
                last_error = Some(e);
                retry_count += 1;
                tokio::time::sleep(std::time::Duration::from_millis(100 * (retry_count as u64)))
                    .await;
            }
        }
    }

    // If we get here, all retries failed
    Err(AppError::DatabaseError(last_error.unwrap_or_else(|| {
        sqlx::Error::Protocol("restore retry failed".into())
    })))
}

#[delete("/api/backups/{filename}")]
pub async fn delete_backup(
    _admin: AdminUser,
    filename: web::Path<String>,
) -> Result<HttpResponse, AppError> {
    if !is_safe_backup_filename(&filename) {
        return Err(AppError::BadRequest("Invalid backup filename".to_string()));
    }

    backup::delete_backup(&filename).await.map_err(|e| {
        error!("Failed to delete backup: {}", e);
        AppError::InternalError(e.to_string())
    })?;
    Ok(HttpResponse::Ok().json(json!({
        "message": "Backup deleted successfully"
    })))
}

#[get("/api/progress/exercise/{exercise_type}")]
pub async fn get_exercise_progress(
    user: CurrentUser,
    db: web::Data<Database>,
    exercise_type: web::Path<String>,
) -> Result<HttpResponse, AppError> {
    let decoded_type = urlencoding::decode(&exercise_type)
        .map_err(|e| AppError::BadRequest(format!("Invalid exercise type: {}", e)))?
        .into_owned();
    let progress = progress::get_exercise_progress(db.get_ref(), user.0.id, &decoded_type).await?;
    Ok(HttpResponse::Ok().json(progress))
}

#[get("/api/progress/workout-stats")]
pub async fn get_workout_stats(
    user: CurrentUser,
    db: web::Data<Database>,
) -> Result<HttpResponse, AppError> {
    let stats = progress::get_workout_stats(db.get_ref(), user.0.id).await?;
    Ok(HttpResponse::Ok().json(stats))
}

#[get("/api/progress/volume")]
pub async fn get_volume_stats(
    user: CurrentUser,
    db: web::Data<Database>,
    query: web::Query<ExerciseTypeQuery>,
) -> Result<HttpResponse, AppError> {
    let decoded_type = urlencoding::decode(&query.exercise_type)
        .map_err(|e| AppError::BadRequest(format!("Invalid exercise type: {}", e)))?
        .into_owned();
    let stats = progress::get_volume_stats(db.get_ref(), user.0.id, &decoded_type).await?;
    Ok(HttpResponse::Ok().json(stats))
}

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(health_check)
        .service(auth::login)
        .service(auth::logout)
        .service(auth::change_password)
        .service(auth::me)
        .service(admin::list_users)
        .service(admin::create_user)
        .service(admin::disable_user)
        .service(admin::reset_user_password)
        .service(admin::delete_user)
        .service(create_workout)
        .service(end_workout)
        .service(update_workout_times)
        .service(get_workout)
        .service(get_workouts)
        .service(create_exercise)
        .service(end_exercise)
        .service(create_set)
        .service(replace_sets)
        .service(write_logs)
        .service(get_exercise_types)
        .service(get_last_exercise_data)
        .service(cancel_exercise)
        .service(cancel_workout)
        .service(list_backups)
        .service(create_manual_backup)
        .service(restore_backup)
        .service(delete_backup)
        .service(get_exercise_progress)
        .service(get_workout_stats)
        .service(get_volume_stats);
}

fn is_safe_backup_filename(filename: &str) -> bool {
    if filename.is_empty() || filename.len() > 200 {
        return false;
    }
    if !filename.ends_with(".tar.gz") {
        return false;
    }
    if filename.contains('/') || filename.contains('\\') || filename.contains("..") {
        return false;
    }
    filename
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || matches!(c, '.' | '-' | '_'))
}
