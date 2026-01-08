use super::Database;
use crate::{errors::AppError, models::*};
use chrono::{DateTime, Utc};
use log::{debug, error, info};

impl Database {
    pub async fn create_workout(&self, req: &CreateWorkoutRequest) -> Result<i64, AppError> {
        debug!(target: "database", "Creating new workout for date: {}", req.date);

        let pool = self.pool().await;
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

    pub async fn update_workout_end_time(
        &self,
        id: i64,
        end_time: DateTime<Utc>,
        notes: Option<String>,
        feedback: Option<String>,
    ) -> Result<(), AppError> {
        debug!(target: "database", "Updating workout #{} end time to {} with feedback", id, end_time);

        let pool = self.pool().await;
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

    pub async fn update_workout_times(
        &self,
        id: i64,
        start_time: DateTime<Utc>,
        end_time: DateTime<Utc>,
    ) -> Result<(), AppError> {
        debug!(
            target: "database",
            "Updating workout #{} times: start={} end={}",
            id,
            start_time,
            end_time
        );

        let pool = self.pool().await;
        sqlx::query!(
            r#"
            UPDATE workouts
            SET date = ?, start_time = ?, end_time = ?
            WHERE id = ?
            "#,
            start_time,
            start_time,
            end_time,
            id,
        )
        .execute(&pool)
        .await
        .map_err(|e| {
            error!(target: "database", "Failed to update workout times: {}", e);
            AppError::DatabaseError(e)
        })?;

        info!(target: "database", "Updated workout #{} times", id);
        Ok(())
    }

    pub async fn get_workout(&self, id: i64) -> Result<Workout, AppError> {
        debug!(target: "database", "Fetching workout #{}", id);

        let pool = self.pool().await;
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

    pub async fn get_workouts(&self) -> Result<Vec<Workout>, AppError> {
        debug!(target: "database", "Fetching all workouts");

        let pool = self.pool().await;
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

        let workouts: Vec<Workout> = rows
            .into_iter()
            .map(|row| Workout {
                id: Some(row.id),
                date: row.date,
                start_time: row.start_time,
                end_time: row.end_time,
                notes: row.notes,
                feedback: row.feedback,
            })
            .collect();

        info!(target: "database", "Retrieved {} workouts", workouts.len());
        Ok(workouts)
    }

    pub async fn delete_workout(&self, id: i64) -> Result<(), AppError> {
        debug!(target: "database", "Deleting workout #{}", id);

        let pool = self.pool().await;
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
}
