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
    idempotency_key: Option<&str>,
) -> Result<i64, AppError> {
    use crate::db::idempotency::KIND_EXERCISE;

    // A retried offline-sync create (same key) returns the original exercise rather
    // than creating a duplicate carrying a duplicated set list (F-HIGH-3).
    if let Some(key) = idempotency_key {
        if let Some(existing) = db.lookup_idempotent(user_id, KIND_EXERCISE, key).await? {
            return Ok(existing);
        }
    }

    let id = db.create_exercise(user_id, workout_id, request).await?;

    if let Some(key) = idempotency_key {
        let authoritative = db.record_idempotent(user_id, KIND_EXERCISE, key, id).await?;
        if authoritative != id {
            // A concurrent request with the same key won the race — drop our
            // duplicate and return the winner.
            let _ = db.delete_exercise(user_id, id).await;
            return Ok(authoritative);
        }
    }

    Ok(id)
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
    exclude_workout_id: Option<i64>,
) -> Result<Option<LastExerciseData>, AppError> {
    let data = db
        .get_last_exercise_data(user_id, exercise_type, exclude_workout_id)
        .await?;
    Ok(data.map(|(exercise, sets)| LastExerciseData { exercise, sets }))
}

pub async fn delete_exercise(
    db: &Database,
    user_id: i64,
    exercise_id: i64,
) -> Result<(), AppError> {
    db.delete_exercise(user_id, exercise_id).await
}
