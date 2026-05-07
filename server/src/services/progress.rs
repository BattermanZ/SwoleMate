use crate::{db::Database, errors::AppError, models::Exercise};
use serde::Serialize;
use serde_json::{json, Value};

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

pub async fn get_workout_stats(db: &Database, user_id: i64) -> Result<Value, AppError> {
    let mut stats = db.get_workout_stats(user_id).await?;
    let frequency = db.get_calendar_workout_frequency(user_id).await?;

    if let Some(object) = stats.as_object_mut() {
        object.insert(
            "workout_frequency".to_string(),
            json!({
                "average_per_week": frequency.average_per_week,
                "trend": frequency.trend
            }),
        );
    }

    Ok(stats)
}

pub async fn get_progress_overview(
    db: &Database,
    user_id: i64,
    timezone_offset_minutes: i64,
) -> Result<Value, AppError> {
    db.get_progress_overview(user_id, timezone_offset_minutes)
        .await
}

pub async fn get_volume_stats(
    db: &Database,
    user_id: i64,
    exercise_type: &str,
) -> Result<serde_json::Value, AppError> {
    db.get_volume_stats(user_id, exercise_type).await
}
