use actix_web::{web, HttpResponse, get, post};
use serde_json::json;
use crate::{models::*, errors::AppError, db::Database};

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

#[post("/workouts/{workout_id}/exercises")]
pub async fn create_exercise(
    db: web::Data<Database>,
    workout_id: web::Path<i64>,
    exercise: web::Json<Exercise>,
) -> Result<HttpResponse, AppError> {
    let mut exercise = exercise.into_inner();
    exercise.workout_id = *workout_id;
    
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
    set: web::Json<Set>,
) -> Result<HttpResponse, AppError> {
    let mut set = set.into_inner();
    set.exercise_id = *exercise_id;
    
    let set_id = db.create_set(&set).await?;
    Ok(HttpResponse::Created().json(json!({
        "id": set_id,
        "message": "Set created successfully"
    })))
}

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(health_check)
        .service(create_workout)
        .service(get_workout)
        .service(create_exercise)
        .service(create_set);
} 