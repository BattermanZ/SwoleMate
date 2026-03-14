use crate::{
    db::Database,
    errors::AppError,
    models::{
        CreateWorkoutRequest, Exercise, Set, UpdateWorkoutRequest, UpdateWorkoutTimesRequest,
        Workout,
    },
};
use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct WorkoutWithSets {
    pub exercise: Exercise,
    pub sets: Vec<Set>,
}

#[derive(Debug, Serialize)]
pub struct WorkoutDetail {
    pub workout: Workout,
    pub exercises: Vec<WorkoutWithSets>,
}

pub async fn create_workout(
    db: &Database,
    user_id: i64,
    request: &CreateWorkoutRequest,
) -> Result<i64, AppError> {
    db.create_workout(user_id, request).await
}

pub async fn end_workout(
    db: &Database,
    user_id: i64,
    workout_id: i64,
    request: &UpdateWorkoutRequest,
) -> Result<(), AppError> {
    db.update_workout_end_time(
        user_id,
        workout_id,
        request.end_time,
        request.notes.clone(),
        request.feedback.clone(),
    )
    .await
}

pub async fn update_workout_times(
    db: &Database,
    user_id: i64,
    workout_id: i64,
    request: &UpdateWorkoutTimesRequest,
) -> Result<(), AppError> {
    db.update_workout_times(
        user_id,
        workout_id,
        request.start_time,
        request.end_time,
        request.notes.clone(),
        request.feedback.clone(),
    )
    .await
}

pub async fn get_workout_detail(
    db: &Database,
    user_id: i64,
    workout_id: i64,
) -> Result<WorkoutDetail, AppError> {
    let workout = db.get_workout(user_id, workout_id).await?;
    let exercises = db.get_exercises_for_workout(user_id, workout_id).await?;

    let mut exercise_details = Vec::with_capacity(exercises.len());
    for exercise in exercises {
        let exercise_id = exercise
            .id
            .ok_or_else(|| AppError::InternalError("Exercise missing id".to_string()))?;
        let sets = db.get_sets_for_exercise(user_id, exercise_id).await?;
        exercise_details.push(WorkoutWithSets { exercise, sets });
    }

    Ok(WorkoutDetail {
        workout,
        exercises: exercise_details,
    })
}

pub async fn list_workouts(db: &Database, user_id: i64) -> Result<Vec<Workout>, AppError> {
    db.get_workouts(user_id).await
}

pub async fn delete_workout(db: &Database, user_id: i64, workout_id: i64) -> Result<(), AppError> {
    db.delete_workout(user_id, workout_id).await
}
