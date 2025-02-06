use actix_web::{web, HttpResponse, get, post, put, delete};
use serde_json::json;
use crate::{models::*, errors::AppError, db::Database};
use serde::Deserialize;
use std::fs;
use std::path::Path;
use std::io::Write;
use log::error;
use crate::models;
use urlencoding;

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
    db.update_workout_end_time(*id, end_req.end_time, end_req.notes.clone(), end_req.feedback.clone()).await?;
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
pub async fn get_workouts(
    db: web::Data<Database>,
) -> Result<HttpResponse, AppError> {
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
    db.update_exercise_end_time(*id, end_req.end_time, end_req.notes.clone()).await?;
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
        .service(cancel_workout);
} 