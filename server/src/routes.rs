use crate::backup::{self, BackupType};
use crate::models;
use crate::{db::Database, errors::AppError, models::*};
use actix_web::{delete, get, post, put, web, HttpResponse};
use log::error;
use serde::Deserialize;
use serde_json::json;
use std::fs;
use std::io::Write;
use std::path::Path;

#[derive(Debug, Deserialize)]
pub struct CreateSetRequest {
    pub reps: i64,
    pub weight: f64,
    pub notes: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateWorkoutRequest {
    pub end_time: chrono::DateTime<chrono::Utc>,
    pub notes: Option<String>,
    pub feedback: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateExerciseRequest {
    pub end_time: chrono::DateTime<chrono::Utc>,
    pub notes: Option<String>,
}

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
    db: web::Data<Database>,
    workout_req: web::Json<models::CreateWorkoutRequest>,
) -> Result<HttpResponse, AppError> {
    let workout_id = db.create_workout(&workout_req.0).await?;
    Ok(HttpResponse::Created().json(json!({
        "id": workout_id,
        "message": "Workout created successfully"
    })))
}

#[put("/api/workouts/{id}/end")]
pub async fn end_workout(
    db: web::Data<Database>,
    id: web::Path<i64>,
    end_req: web::Json<UpdateWorkoutRequest>,
) -> Result<HttpResponse, AppError> {
    db.update_workout_end_time(
        *id,
        end_req.end_time,
        end_req.notes.clone(),
        end_req.feedback.clone(),
    )
    .await?;
    Ok(HttpResponse::Ok().json(json!({
        "message": "Workout ended successfully"
    })))
}

#[get("/api/workouts/{id}")]
pub async fn get_workout(
    db: web::Data<Database>,
    id: web::Path<i64>,
) -> Result<HttpResponse, AppError> {
    let workout = db.get_workout(*id).await?;
    let exercises = db.get_exercises_for_workout(*id).await?;

    let mut exercises_with_sets = Vec::new();
    for exercise in exercises {
        let sets = db.get_sets_for_exercise(exercise.id.unwrap()).await?;
        exercises_with_sets.push(json!({
            "exercise": exercise,
            "sets": sets
        }));
    }

    Ok(HttpResponse::Ok().json(json!({
        "workout": workout,
        "exercises": exercises_with_sets
    })))
}

#[get("/api/workouts")]
pub async fn get_workouts(db: web::Data<Database>) -> Result<HttpResponse, AppError> {
    let workouts = db.get_workouts().await?;
    Ok(HttpResponse::Ok().json(workouts))
}

#[post("/api/workouts/{workout_id}/exercises")]
pub async fn create_exercise(
    db: web::Data<Database>,
    workout_id: web::Path<i64>,
    exercise_req: web::Json<CreateExerciseRequest>,
) -> Result<HttpResponse, AppError> {
    let exercise = Exercise {
        id: None,
        workout_id: *workout_id,
        exercise_type: exercise_req.exercise_type.clone(),
        start_time: exercise_req.start_time,
        end_time: exercise_req.start_time, // Will be updated later
        notes: exercise_req.notes.clone(),
    };

    let exercise_id = db.create_exercise(&exercise).await?;
    Ok(HttpResponse::Created().json(json!({
        "id": exercise_id,
        "message": "Exercise created successfully"
    })))
}

#[put("/api/exercises/{id}/end")]
pub async fn end_exercise(
    db: web::Data<Database>,
    id: web::Path<i64>,
    end_req: web::Json<UpdateExerciseRequest>,
) -> Result<HttpResponse, AppError> {
    db.update_exercise_end_time(*id, end_req.end_time, end_req.notes.clone())
        .await?;
    Ok(HttpResponse::Ok().json(json!({
        "message": "Exercise ended successfully"
    })))
}

#[post("/api/exercises/{exercise_id}/sets")]
pub async fn create_set(
    db: web::Data<Database>,
    exercise_id: web::Path<i64>,
    set_req: web::Json<CreateSetRequest>,
) -> Result<HttpResponse, AppError> {
    let set = Set {
        id: None,
        exercise_id: *exercise_id,
        reps: set_req.reps,
        weight: set_req.weight,
        notes: set_req.notes.clone(),
    };

    let set_id = db.create_set(&set).await?;
    Ok(HttpResponse::Created().json(json!({
        "id": set_id,
        "message": "Set created successfully"
    })))
}

#[post("/api/logs/init")]
pub async fn init_logs_directory() -> HttpResponse {
    let logs_dir = Path::new("logs");
    if !logs_dir.exists() {
        if let Err(e) = fs::create_dir(logs_dir) {
            error!("Failed to create logs directory: {}", e);
            return HttpResponse::InternalServerError().json(json!({
                "error": "Failed to create logs directory"
            }));
        }
    }
    HttpResponse::Ok().json(json!({ "status": "ok" }))
}

#[post("/api/logs")]
pub async fn write_logs(logs: web::Json<Vec<serde_json::Value>>) -> HttpResponse {
    let client_log_path = Path::new("logs/client.log");
    let file = fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(client_log_path);

    match file {
        Ok(mut file) => {
            for log in logs.iter() {
                if let Err(e) = writeln!(file, "{}", serde_json::to_string(log).unwrap()) {
                    error!("Failed to write client log: {}", e);
                    return HttpResponse::InternalServerError().json(json!({
                        "error": "Failed to write logs"
                    }));
                }
            }
            HttpResponse::Ok().json(json!({ "status": "ok" }))
        }
        Err(e) => {
            error!("Failed to open client log file: {}", e);
            HttpResponse::InternalServerError().json(json!({
                "error": "Failed to open log file"
            }))
        }
    }
}

#[get("/api/exercises/types")]
pub async fn get_exercise_types(db: web::Data<Database>) -> Result<HttpResponse, AppError> {
    let types = db.get_unique_exercise_types().await?;
    Ok(HttpResponse::Ok().json(types))
}

#[get("/api/exercises/last/{exercise_type}")]
pub async fn get_last_exercise_data(
    db: web::Data<Database>,
    exercise_type: web::Path<String>,
) -> Result<HttpResponse, AppError> {
    let decoded_type = urlencoding::decode(&exercise_type)
        .map_err(|e| AppError::BadRequest(format!("Invalid exercise type: {}", e)))?
        .into_owned();
    let data = db.get_last_exercise_data(&decoded_type).await?;
    Ok(HttpResponse::Ok().json(data))
}

#[delete("/api/exercises/{id}")]
pub async fn cancel_exercise(
    db: web::Data<Database>,
    id: web::Path<i64>,
) -> Result<HttpResponse, AppError> {
    db.delete_exercise(*id).await?;
    Ok(HttpResponse::Ok().json(json!({
        "message": "Exercise canceled successfully"
    })))
}

#[delete("/api/workouts/{id}")]
pub async fn cancel_workout(
    db: web::Data<Database>,
    id: web::Path<i64>,
) -> Result<HttpResponse, AppError> {
    db.delete_workout(*id).await?;
    Ok(HttpResponse::Ok().json(json!({
        "message": "Workout canceled successfully"
    })))
}

#[get("/api/backups")]
pub async fn list_backups() -> Result<HttpResponse, AppError> {
    let backups = backup::list_backups().await.map_err(|e| {
        error!("Failed to list backups: {}", e);
        AppError::InternalError(e.to_string())
    })?;
    Ok(HttpResponse::Ok().json(backups))
}

#[post("/api/backups")]
pub async fn create_manual_backup() -> Result<HttpResponse, AppError> {
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
    filename: web::Path<String>,
    db: web::Data<Database>,
) -> Result<HttpResponse, AppError> {
    // Close all existing connections in the pool
    let current_pool = db.get_pool();
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
        match sqlx::SqlitePool::connect(&db_url).await {
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
                for pragma in [
                    "PRAGMA journal_mode = WAL",
                    "PRAGMA synchronous = NORMAL",
                    "PRAGMA foreign_keys = ON",
                    "PRAGMA busy_timeout = 5000",
                ] {
                    if let Err(e) = sqlx::query(pragma).execute(&new_pool).await {
                        error!("Failed to set pragma {}: {}", pragma, e);
                        new_pool.close().await;
                        retry_count += 1;
                        tokio::time::sleep(std::time::Duration::from_millis(
                            100 * (retry_count as u64),
                        ))
                        .await;
                        continue;
                    }
                }

                // Update the database instance with the new pool
                db.update_pool(new_pool);
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
    Err(AppError::DatabaseError(last_error.unwrap()))
}

#[delete("/api/backups/{filename}")]
pub async fn delete_backup(filename: web::Path<String>) -> Result<HttpResponse, AppError> {
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
    db: web::Data<Database>,
    exercise_type: web::Path<String>,
) -> Result<HttpResponse, AppError> {
    let decoded_type = urlencoding::decode(&exercise_type)
        .map_err(|e| AppError::BadRequest(format!("Invalid exercise type: {}", e)))?
        .into_owned();
    let progress = db.get_exercise_progress(&decoded_type).await?;
    let progress = progress
        .into_iter()
        .map(|(exercise, sets)| {
            json!({
                "exercise": exercise,
                "sets": sets,
            })
        })
        .collect::<Vec<_>>();
    Ok(HttpResponse::Ok().json(progress))
}

#[get("/api/progress/workout-stats")]
pub async fn get_workout_stats(db: web::Data<Database>) -> Result<HttpResponse, AppError> {
    let stats = db.get_workout_stats().await?;
    Ok(HttpResponse::Ok().json(stats))
}

#[get("/api/progress/volume")]
pub async fn get_volume_stats(
    db: web::Data<Database>,
    query: web::Query<ExerciseTypeQuery>,
) -> Result<HttpResponse, AppError> {
    let decoded_type = urlencoding::decode(&query.exercise_type)
        .map_err(|e| AppError::BadRequest(format!("Invalid exercise type: {}", e)))?
        .into_owned();
    let stats = db.get_volume_stats(&decoded_type).await?;
    Ok(HttpResponse::Ok().json(stats))
}

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(health_check)
        .service(create_workout)
        .service(end_workout)
        .service(get_workout)
        .service(get_workouts)
        .service(create_exercise)
        .service(end_exercise)
        .service(create_set)
        .service(init_logs_directory)
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
