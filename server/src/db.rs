use sqlx::{Pool, Sqlite};
use crate::{models::*, errors::AppError};
use log::{debug, error, info};
use chrono::{DateTime, Utc};
use std::sync::{Arc, Mutex};
use serde_json::json;

#[derive(Clone)]
pub struct Database {
    pool: Arc<Mutex<Pool<Sqlite>>>,
}

impl Database {
    pub fn new(pool: Pool<Sqlite>) -> Self {
        info!(target: "database", "Database connection pool initialized");
        Self { 
            pool: Arc::new(Mutex::new(pool))
        }
    }

    pub fn get_pool(&self) -> Pool<Sqlite> {
        self.pool.lock().unwrap().clone()
    }

    pub fn update_pool(&self, new_pool: Pool<Sqlite>) {
        let mut pool = self.pool.lock().unwrap();
        *pool = new_pool;
        info!(target: "database", "Database connection pool updated");
    }

    pub async fn create_workout(&self, req: &CreateWorkoutRequest) -> Result<i64, AppError> {
        debug!(target: "database", "Creating new workout for date: {}", req.date);
        
        let pool = self.get_pool();
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
        .fetch_one(&pool)
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
        
        let pool = self.get_pool();
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
        .execute(&pool)
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
        
        let pool = self.get_pool();
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
        .fetch_one(&pool)
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
        
        let pool = self.get_pool();
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
        .execute(&pool)
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
        
        let pool = self.get_pool();
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
        .fetch_one(&pool)
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
        
        let pool = self.get_pool();
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
        .fetch_optional(&pool)
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
        
        let pool = self.get_pool();
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
        .fetch_all(&pool)
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
        
        let pool = self.get_pool();
        let sets = sqlx::query_as!(
            Set,
            r#"
            SELECT id, exercise_id, reps, weight, notes
            FROM sets
            WHERE exercise_id = ?
            "#,
            exercise_id
        )
        .fetch_all(&pool)
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
        
        let pool = self.get_pool();
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
        .fetch_all(&pool)
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
        
        let pool = self.get_pool();
        let rows = sqlx::query!(
            r#"
            SELECT DISTINCT exercise_type
            FROM exercises
            ORDER BY exercise_type ASC
            "#
        )
        .fetch_all(&pool)
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
        
        let pool = self.get_pool();
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
        .fetch_optional(&pool)
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

    pub async fn delete_exercise(&self, id: i64) -> Result<(), AppError> {
        debug!(target: "database", "Deleting exercise #{}", id);
        
        let pool = self.get_pool();
        // With CASCADE DELETE, we only need to delete the exercise
        // and all related sets will be automatically deleted
        sqlx::query!(
            r#"
            DELETE FROM exercises
            WHERE id = ?
            "#,
            id,
        )
        .execute(&pool)
        .await
        .map_err(|e| {
            error!(target: "database", "Failed to delete exercise: {}", e);
            AppError::DatabaseError(e)
        })?;

        info!(target: "database", "Deleted exercise #{} and its sets", id);
        Ok(())
    }

    pub async fn delete_workout(&self, id: i64) -> Result<(), AppError> {
        debug!(target: "database", "Deleting workout #{}", id);
        
        let pool = self.get_pool();
        // With CASCADE DELETE, we only need to delete the workout
        // and all related exercises and sets will be automatically deleted
        sqlx::query!(
            r#"
            DELETE FROM workouts
            WHERE id = ?
            "#,
            id,
        )
        .execute(&pool)
        .await
        .map_err(|e| {
            error!(target: "database", "Failed to delete workout: {}", e);
            AppError::DatabaseError(e)
        })?;

        info!(target: "database", "Deleted workout #{} and all its exercises and sets", id);
        Ok(())
    }

    pub async fn get_exercise_progress(&self, exercise_type: &str) -> Result<Vec<(Exercise, Vec<Set>)>, AppError> {
        debug!(target: "database", "Fetching progress data for exercise type: {}", exercise_type);
        
        let pool = self.get_pool();
        let exercises = sqlx::query_as!(
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
            ORDER BY start_time ASC
            "#,
            exercise_type
        )
        .fetch_all(&pool)
        .await
        .map_err(|e| {
            error!(target: "database", "Failed to fetch exercise progress: {}", e);
            AppError::DatabaseError(e)
        })?;

        let mut result = Vec::new();
        for exercise in exercises {
            let sets = self.get_sets_for_exercise(exercise.id.unwrap()).await?;
            result.push((exercise, sets));
        }

        debug!(target: "database", "Found {} exercises for progress data", result.len());
        Ok(result)
    }

    pub async fn get_workout_stats(&self) -> Result<serde_json::Value, AppError> {
        debug!(target: "database", "Calculating workout statistics");
        
        let pool = self.get_pool();
        let stats = sqlx::query!(
            r#"
            SELECT 
                COUNT(*) as total_workouts,
                AVG(ROUND((julianday(end_time) - julianday(start_time)) * 24 * 60)) as avg_duration,
                (
                    SELECT COUNT(*)
                    FROM workouts w2
                    WHERE feedback = '😊'
                ) as good_workouts,
                (
                    SELECT COUNT(*)
                    FROM workouts w3
                    WHERE feedback = '😐'
                ) as neutral_workouts,
                (
                    SELECT COUNT(*)
                    FROM workouts w4
                    WHERE feedback = '😞'
                ) as bad_workouts
            FROM workouts w1
            "#
        )
        .fetch_one(&pool)
        .await
        .map_err(|e| {
            error!(target: "database", "Failed to fetch workout stats: {}", e);
            AppError::DatabaseError(e)
        })?;

        Ok(json!({
            "total_workouts": stats.total_workouts,
            "average_duration_minutes": stats.avg_duration,
            "feedback_distribution": {
                "good": stats.good_workouts,
                "neutral": stats.neutral_workouts,
                "bad": stats.bad_workouts
            }
        }))
    }

    pub async fn get_volume_stats(&self, exercise_type: &str) -> Result<serde_json::Value, AppError> {
        debug!(target: "database", "Calculating volume statistics for {}", exercise_type);
        
        let pool = self.get_pool();
        let weekly_volume = sqlx::query!(
            r#"
            SELECT 
                strftime('%Y-%W', e.start_time) as week,
                SUM(s.reps * s.weight) as total_volume,
                MAX(s.weight) as max_weight,
                SUM(s.reps) as total_reps,
                COUNT(DISTINCT e.id) as sessions
            FROM exercises e
            JOIN sets s ON e.id = s.exercise_id
            WHERE LOWER(e.exercise_type) = LOWER(?)
            GROUP BY strftime('%Y-%W', e.start_time)
            ORDER BY week ASC
            "#,
            exercise_type
        )
        .fetch_all(&pool)
        .await
        .map_err(|e| {
            error!(target: "database", "Failed to fetch volume stats: {}", e);
            AppError::DatabaseError(e)
        })?;

        let personal_records = sqlx::query!(
            r#"
            SELECT 
                MAX(s.weight) as max_weight,
                s.reps,
                e.start_time as "achieved_at: DateTime<Utc>"
            FROM exercises e
            JOIN sets s ON e.id = s.exercise_id
            WHERE LOWER(e.exercise_type) = LOWER(?)
            GROUP BY s.reps
            ORDER BY s.reps ASC
            "#,
            exercise_type
        )
        .fetch_all(&pool)
        .await
        .map_err(|e| {
            error!(target: "database", "Failed to fetch PRs: {}", e);
            AppError::DatabaseError(e)
        })?;

        Ok(json!({
            "weekly_volume": weekly_volume.iter().map(|row| {
                json!({
                    "week": row.week,
                    "total_volume": row.total_volume,
                    "max_weight": row.max_weight,
                    "total_reps": row.total_reps,
                    "sessions": row.sessions
                })
            }).collect::<Vec<_>>(),
            "personal_records": personal_records.iter().map(|row| {
                json!({
                    "reps": row.reps,
                    "weight": row.max_weight,
                    "achieved_at": row.achieved_at
                })
            }).collect::<Vec<_>>()
        }))
    }
} 