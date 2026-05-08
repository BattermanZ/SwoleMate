use crate::{
    db::Database,
    errors::AppError,
    models::{CreateExerciseRequest, CreateSetRequest, Exercise, Set, UpdateExerciseRequest},
};
use serde::Serialize;

const MAX_SETS_PER_EXERCISE_REPLACE: usize = 100;

#[derive(Debug, Serialize)]
pub struct LastExerciseData {
    pub exercise: Exercise,
    pub sets: Vec<Set>,
}

pub async fn create_exercise(
    db: &Database,
    user_id: i64,
    workout_id: i64,
    request: &CreateExerciseRequest,
) -> Result<i64, AppError> {
    db.create_exercise(user_id, workout_id, request).await
}

pub async fn end_exercise(
    db: &Database,
    user_id: i64,
    exercise_id: i64,
    request: &UpdateExerciseRequest,
) -> Result<(), AppError> {
    db.update_exercise(user_id, exercise_id, request).await
}

pub async fn create_set(
    db: &Database,
    user_id: i64,
    exercise_id: i64,
    request: &CreateSetRequest,
) -> Result<Set, AppError> {
    db.create_set(user_id, exercise_id, request).await
}

pub async fn replace_sets(
    db: &Database,
    user_id: i64,
    exercise_id: i64,
    requests: &[CreateSetRequest],
) -> Result<Vec<Set>, AppError> {
    if requests.len() > MAX_SETS_PER_EXERCISE_REPLACE {
        return Err(AppError::BadRequest(format!(
            "sets must have at most {MAX_SETS_PER_EXERCISE_REPLACE} items"
        )));
    }
    db.replace_sets_for_exercise(user_id, exercise_id, requests)
        .await
}

pub async fn list_exercise_types(db: &Database, user_id: i64) -> Result<Vec<String>, AppError> {
    db.get_unique_exercise_types(user_id).await
}

pub async fn get_last_exercise_data(
    db: &Database,
    user_id: i64,
    exercise_type: &str,
) -> Result<Option<LastExerciseData>, AppError> {
    let data = db.get_last_exercise_data(user_id, exercise_type).await?;
    Ok(data.map(|(exercise, sets)| LastExerciseData { exercise, sets }))
}

pub async fn delete_exercise(
    db: &Database,
    user_id: i64,
    exercise_id: i64,
) -> Result<(), AppError> {
    db.delete_exercise(user_id, exercise_id).await
}
