use sqlx::{Pool, Sqlite};
use crate::{models::*, errors::AppError};
use log::{debug, error, info};
use chrono::{DateTime, Utc};

#[derive(Clone)]
pub struct Database {
    pool: Pool<Sqlite>,
}

impl Database {
    pub fn new(pool: Pool<Sqlite>) -> Self {
        info!(target: "database", "Database connection pool initialized");
        Self { pool }
    }

    pub async fn create_workout(&self, req: &CreateWorkoutRequest) -> Result<i64, AppError> {
        debug!(target: "database", "Creating new workout for date: {}", req.date);
        
        let result = sqlx::query!(
            r#"
            INSERT INTO workouts (date, start_time, end_time, notes)
            VALUES (?, ?, ?, ?)
            RETURNING id
            "#,
            req.date,
            req.start_time,
            req.start_time, // Initially set end_time to start_time
            req.notes,
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| {
            error!(target: "database", "Failed to create workout: {}", e);
            AppError::DatabaseError(e)
        })?;

        info!(target: "database", "Created workout #{} for date {}", result.id, req.date);
        Ok(result.id)
    }

    pub async fn update_workout_end_time(&self, id: i64, end_time: DateTime<Utc>, notes: Option<String>, feedback: Option<String>) -> Result<(), AppError> {
        debug!(target: "database", "Updating workout #{} end time to {} with feedback", id, end_time);
        
        sqlx::query!(
            r#"
            UPDATE workouts
            SET end_time = ?, notes = ?, feedback = ?
            WHERE id = ?
            "#,
            end_time,
            notes,
            feedback,
            id,
        )
        .execute(&self.pool)
        .await
        .map_err(|e| {
            error!(target: "database", "Failed to update workout end time: {}", e);
            AppError::DatabaseError(e)
        })?;

        info!(target: "database", "Updated workout #{} end time and feedback", id);
        Ok(())
    }

    pub async fn create_exercise(&self, exercise: &Exercise) -> Result<i64, AppError> {
        debug!(target: "database", "Creating exercise '{}' for workout #{}", 
            exercise.exercise_type, exercise.workout_id);
        
        let result = sqlx::query!(
            r#"
            INSERT INTO exercises (workout_id, exercise_type, start_time, end_time, notes)
            VALUES (?, ?, ?, ?, ?)
            RETURNING id
            "#,
            exercise.workout_id,
            exercise.exercise_type,
            exercise.start_time,
            exercise.end_time,
            exercise.notes,
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| {
            error!(target: "database", "Failed to create exercise: {}", e);
            AppError::DatabaseError(e)
        })?;

        info!(target: "database", "Created exercise #{}", result.id);
        Ok(result.id)
    }

    pub async fn update_exercise_end_time(&self, id: i64, end_time: DateTime<Utc>, notes: Option<String>) -> Result<(), AppError> {
        debug!(target: "database", "Updating exercise #{} end time to {} with notes", id, end_time);
        
        sqlx::query!(
            r#"
            UPDATE exercises
            SET end_time = ?, notes = ?
            WHERE id = ?
            "#,
            end_time,
            notes,
            id,
        )
        .execute(&self.pool)
        .await
        .map_err(|e| {
            error!(target: "database", "Failed to update exercise end time: {}", e);
            AppError::DatabaseError(e)
        })?;

        info!(target: "database", "Updated exercise #{} end time and notes", id);
        Ok(())
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
            SELECT 
                id,
                date as "date: DateTime<Utc>",
                start_time as "start_time: DateTime<Utc>",
                end_time as "end_time: DateTime<Utc>",
                notes,
                feedback
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
            date: result.date,
            start_time: result.start_time,
            end_time: result.end_time,
            notes: result.notes,
            feedback: result.feedback,
        })
    }

    pub async fn get_exercises_for_workout(&self, workout_id: i64) -> Result<Vec<Exercise>, AppError> {
        debug!(target: "database", "Fetching exercises for workout #{}", workout_id);
        
        let rows = sqlx::query!(
            r#"
            SELECT 
                id as "id?",
                workout_id,
                exercise_type,
                start_time as "start_time: DateTime<Utc>",
                end_time as "end_time: DateTime<Utc>",
                notes
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

        let exercises: Vec<Exercise> = rows.into_iter().map(|row| Exercise {
            id: Some(row.id.expect("Exercise ID should not be null")),
            workout_id: row.workout_id,
            exercise_type: row.exercise_type,
            start_time: row.start_time,
            end_time: row.end_time,
            notes: row.notes,
        }).collect();

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
            SELECT 
                id as "id!",
                date as "date: DateTime<Utc>",
                start_time as "start_time: DateTime<Utc>",
                end_time as "end_time: DateTime<Utc>",
                notes,
                feedback
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
            id: Some(row.id),
            date: row.date,
            start_time: row.start_time,
            end_time: row.end_time,
            notes: row.notes,
            feedback: row.feedback,
        }).collect();

        info!(target: "database", "Retrieved {} workouts", workouts.len());
        Ok(workouts)
    }

    pub async fn get_unique_exercise_types(&self) -> Result<Vec<String>, AppError> {
        debug!(target: "database", "Fetching unique exercise types");
        
        let rows = sqlx::query!(
            r#"
            SELECT DISTINCT exercise_type
            FROM exercises
            ORDER BY exercise_type ASC
            "#
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| {
            error!(target: "database", "Failed to fetch exercise types: {}", e);
            AppError::DatabaseError(e)
        })?;

        let row_count = rows.len();
        let exercise_types = rows.into_iter()
            .map(|row| row.exercise_type)
            .collect();

        debug!(target: "database", "Found {} unique exercise types", row_count);
        Ok(exercise_types)
    }

    pub async fn get_last_exercise_data(&self, exercise_type: &str) -> Result<Option<(Exercise, Vec<Set>)>, AppError> {
        debug!(target: "database", "Fetching last data for exercise type: {}", exercise_type);
        
        let exercise = sqlx::query_as!(
            Exercise,
            r#"
            SELECT 
                id as "id?",
                workout_id,
                exercise_type,
                start_time as "start_time: DateTime<Utc>",
                end_time as "end_time: DateTime<Utc>",
                notes
            FROM exercises
            WHERE LOWER(exercise_type) = LOWER(?)
            ORDER BY start_time DESC
            LIMIT 1
            "#,
            exercise_type
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| {
            error!(target: "database", "Failed to fetch last exercise data: {}", e);
            AppError::DatabaseError(e)
        })?;

        if let Some(exercise) = exercise {
            let sets = self.get_sets_for_exercise(exercise.id.unwrap()).await?;
            Ok(Some((exercise, sets)))
        } else {
            Ok(None)
        }
    }
} 