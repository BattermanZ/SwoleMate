use crate::{db::Database, errors::AppError, models::Exercise};
use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct ExerciseProgressEntry {
    pub exercise: Exercise,
    pub sets: Vec<crate::models::Set>,
}

pub async fn get_exercise_progress(
    db: &Database,
    user_id: i64,
    exercise_type: &str,
) -> Result<Vec<ExerciseProgressEntry>, AppError> {
    let progress = db.get_exercise_progress(user_id, exercise_type).await?;
    Ok(progress
        .into_iter()
        .map(|(exercise, sets)| ExerciseProgressEntry { exercise, sets })
        .collect())
}

pub async fn get_workout_stats(db: &Database, user_id: i64) -> Result<serde_json::Value, AppError> {
    db.get_workout_stats(user_id).await
}

pub async fn get_volume_stats(
    db: &Database,
    user_id: i64,
    exercise_type: &str,
) -> Result<serde_json::Value, AppError> {
    db.get_volume_stats(user_id, exercise_type).await
}
