use sqlx::{Pool, Sqlite};
use crate::{models::*, errors::AppError};
use log::{debug, error};
use chrono::{DateTime, Utc, NaiveDateTime};

#[derive(Clone)]
pub struct Database {
    pool: Pool<Sqlite>,
}

impl Database {
    pub fn new(pool: Pool<Sqlite>) -> Self {
        Self { pool }
    }

    pub async fn create_workout(&self, workout: &Workout) -> Result<i64, AppError> {
        debug!("Creating new workout: {:?}", workout);
        
        let naive_date = workout.date.naive_utc();
        let result = sqlx::query!(
            r#"
            INSERT INTO workouts (date, notes)
            VALUES (?, ?)
            RETURNING id
            "#,
            naive_date,
            workout.notes,
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| {
            error!("Failed to create workout: {}", e);
            AppError::DatabaseError(e)
        })?;

        Ok(result.id)
    }

    pub async fn create_exercise(&self, exercise: &Exercise) -> Result<i64, AppError> {
        debug!("Creating new exercise: {:?}", exercise);
        
        let result = sqlx::query!(
            r#"
            INSERT INTO exercises (workout_id, exercise_type, notes)
            VALUES (?, ?, ?)
            RETURNING id
            "#,
            exercise.workout_id,
            exercise.exercise_type,
            exercise.notes,
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| {
            error!("Failed to create exercise: {}", e);
            AppError::DatabaseError(e)
        })?;

        Ok(result.id)
    }

    pub async fn create_set(&self, set: &Set) -> Result<i64, AppError> {
        debug!("Creating new set: {:?}", set);
        
        let result = sqlx::query!(
            r#"
            INSERT INTO sets (exercise_id, reps, weight, notes)
            VALUES (?, ?, ?, ?)
            RETURNING id
            "#,
            set.exercise_id,
            set.reps,
            set.weight,
            set.notes,
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| {
            error!("Failed to create set: {}", e);
            AppError::DatabaseError(e)
        })?;

        Ok(result.id)
    }

    pub async fn get_workout(&self, id: i64) -> Result<Workout, AppError> {
        debug!("Fetching workout with id: {}", id);
        
        let result = sqlx::query!(
            r#"
            SELECT id, date, notes
            FROM workouts
            WHERE id = ?
            "#,
            id
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| {
            error!("Failed to fetch workout: {}", e);
            AppError::DatabaseError(e)
        })?
        .ok_or_else(|| AppError::NotFound(format!("Workout with id {} not found", id)))?;

        Ok(Workout {
            id: Some(result.id),
            date: DateTime::from_naive_utc_and_offset(
                NaiveDateTime::from(result.date),
                Utc,
            ),
            notes: result.notes,
        })
    }

    pub async fn get_exercises_for_workout(&self, workout_id: i64) -> Result<Vec<Exercise>, AppError> {
        debug!("Fetching exercises for workout: {}", workout_id);
        
        let exercises = sqlx::query_as!(
            Exercise,
            r#"
            SELECT id, workout_id, exercise_type, notes
            FROM exercises
            WHERE workout_id = ?
            "#,
            workout_id
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| {
            error!("Failed to fetch exercises: {}", e);
            AppError::DatabaseError(e)
        })?;

        Ok(exercises)
    }

    pub async fn get_sets_for_exercise(&self, exercise_id: i64) -> Result<Vec<Set>, AppError> {
        debug!("Fetching sets for exercise: {}", exercise_id);
        
        let sets = sqlx::query_as!(
            Set,
            r#"
            SELECT id, exercise_id, reps, weight, notes
            FROM sets
            WHERE exercise_id = ?
            "#,
            exercise_id
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| {
            error!("Failed to fetch sets: {}", e);
            AppError::DatabaseError(e)
        })?;

        Ok(sets)
    }

    pub async fn get_workouts(&self) -> Result<Vec<Workout>, AppError> {
        debug!("Fetching all workouts");
        
        let rows = sqlx::query!(
            r#"
            SELECT id, date, notes
            FROM workouts
            ORDER BY date DESC
            "#
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| {
            error!("Failed to fetch workouts: {}", e);
            AppError::DatabaseError(e)
        })?;

        let workouts = rows.into_iter().map(|row| Workout {
            id: Some(row.id.unwrap()),
            date: DateTime::from_naive_utc_and_offset(
                NaiveDateTime::from(row.date),
                Utc,
            ),
            notes: row.notes,
        }).collect();

        Ok(workouts)
    }
} 