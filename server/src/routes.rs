use actix_web::{web, HttpResponse, get, post};
use serde_json::json;
use crate::{models::*, errors::AppError, db::Database};
use serde::Deserialize;
use std::fs;
use std::path::Path;
use std::io::Write;
use log::error;

#[derive(Debug, Deserialize)]
pub struct CreateExerciseRequest {
    pub exercise_type: String,
    pub notes: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct CreateSetRequest {
    pub reps: i64,
    pub weight: f64,
    pub notes: Option<String>,
}

#[get("/health")]
pub async fn health_check() -> HttpResponse {
    HttpResponse::Ok().json(json!({
        "status": "healthy",
        "timestamp": chrono::Utc::now()
    }))
}

#[post("/workouts")]
pub async fn create_workout(
    db: web::Data<Database>,
    workout: web::Json<Workout>,
) -> Result<HttpResponse, AppError> {
    let workout_id = db.create_workout(&workout).await?;
    Ok(HttpResponse::Created().json(json!({
        "id": workout_id,
        "message": "Workout created successfully"
    })))
}

#[get("/workouts/{id}")]
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

#[get("/workouts")]
pub async fn get_workouts(
    db: web::Data<Database>,
) -> Result<HttpResponse, AppError> {
    let workouts = db.get_workouts().await?;
    Ok(HttpResponse::Ok().json(workouts))
}

#[post("/workouts/{workout_id}/exercises")]
pub async fn create_exercise(
    db: web::Data<Database>,
    workout_id: web::Path<i64>,
    exercise_req: web::Json<CreateExerciseRequest>,
) -> Result<HttpResponse, AppError> {
    let exercise = Exercise {
        id: None,
        workout_id: *workout_id,
        exercise_type: exercise_req.exercise_type.clone(),
        notes: exercise_req.notes.clone(),
    };
    
    let exercise_id = db.create_exercise(&exercise).await?;
    Ok(HttpResponse::Created().json(json!({
        "id": exercise_id,
        "message": "Exercise created successfully"
    })))
}

#[post("/exercises/{exercise_id}/sets")]
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

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(health_check)
        .service(create_workout)
        .service(get_workout)
        .service(get_workouts)
        .service(create_exercise)
        .service(create_set)
        .service(init_logs_directory)
        .service(write_logs);
} 