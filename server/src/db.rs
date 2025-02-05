use sqlx::{Pool, Sqlite};
use crate::{models::*, errors::AppError};
use log::{debug, error, info};
use chrono::{DateTime, Utc, NaiveDateTime};
use serde_json::json;

#[derive(Clone)]
pub struct Database {
    pool: Pool<Sqlite>,
}

impl Database {
    pub fn new(pool: Pool<Sqlite>) -> Self {
        info!(target: "database", "Database connection pool initialized");
        Self { pool }
    }

    pub async fn create_workout(&self, workout: &Workout) -> Result<i64, AppError> {
        debug!(target: "database", "Creating new workout for date: {}", workout.date);
        
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
            error!(target: "database", "Failed to create workout: {}", e);
            AppError::DatabaseError(e)
        })?;

        info!(target: "database", "Created workout #{} for date {}", result.id, workout.date);
        Ok(result.id)
    }

    pub async fn create_exercise(&self, exercise: &Exercise) -> Result<i64, AppError> {
        debug!(target: "database", "Creating exercise '{}' for workout #{}", 
            exercise.exercise_type, exercise.workout_id);
        
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
            error!(target: "database", "Failed to create exercise: {}", e);
            AppError::DatabaseError(e)
        })?;

        debug!(target: "database", "Created exercise #{}", result.id);
        Ok(result.id)
    }

    pub async fn create_set(&self, set: &Set) -> Result<i64, AppError> {
        debug!(target: "database", "Creating set for exercise #{}: {}x{}kg", 
            set.exercise_id, set.reps, set.weight);
        
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
            error!(target: "database", "Failed to create set: {}", e);
            AppError::DatabaseError(e)
        })?;

        debug!(target: "database", "Created set #{}", result.id);
        Ok(result.id)
    }

    pub async fn get_workout(&self, id: i64) -> Result<Workout, AppError> {
        debug!(target: "database", "Fetching workout #{}", id);
        
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
            error!(target: "database", "Failed to fetch workout: {}", e);
            AppError::DatabaseError(e)
        })?
        .ok_or_else(|| AppError::NotFound(format!("Workout #{} not found", id)))?;

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
        debug!(target: "database", "Fetching exercises for workout #{}", workout_id);
        
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
            error!(target: "database", "Failed to fetch exercises: {}", e);
            AppError::DatabaseError(e)
        })?;

        debug!(target: "database", "Found {} exercises for workout #{}", exercises.len(), workout_id);
        Ok(exercises)
    }

    pub async fn get_sets_for_exercise(&self, exercise_id: i64) -> Result<Vec<Set>, AppError> {
        debug!(target: "database", "Fetching sets for exercise #{}", exercise_id);
        
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
            error!(target: "database", "Failed to fetch sets: {}", e);
            AppError::DatabaseError(e)
        })?;

        debug!(target: "database", "Found {} sets for exercise #{}", sets.len(), exercise_id);
        Ok(sets)
    }

    pub async fn get_workouts(&self) -> Result<Vec<Workout>, AppError> {
        debug!(target: "database", "Fetching all workouts");
        
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
            error!(target: "database", "Failed to fetch workouts: {}", e);
            AppError::DatabaseError(e)
        })?;

        let workouts: Vec<Workout> = rows.into_iter().map(|row| Workout {
            id: Some(row.id.unwrap()),
            date: DateTime::from_naive_utc_and_offset(
                NaiveDateTime::from(row.date),
                Utc,
            ),
            notes: row.notes,
        }).collect();

        info!(target: "database", "Retrieved {} workouts", workouts.len());
        Ok(workouts)
    }
} 