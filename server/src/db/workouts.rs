use super::Database;
use crate::{errors::AppError, models::*};
use chrono::{DateTime, Utc};
use log::{debug, error, info};
use sqlx::{Executor, Sqlite};

impl Database {
    pub(crate) async fn touch_workout_activity_at<'e, E>(
        &self,
        user_id: i64,
        workout_id: i64,
        at: DateTime<Utc>,
        executor: E,
    ) -> Result<(), AppError>
    where
        E: Executor<'e, Database = Sqlite>,
    {
        sqlx::query!(
            r#"
            UPDATE workouts
            SET last_activity_time = ?
            WHERE id = ? AND user_id = ?
            "#,
            at,
            workout_id,
            user_id
        )
        .execute(executor)
        .await
        .map_err(|e| {
            error!(target: "database", "Failed to touch workout activity: {}", e);
            AppError::DatabaseError(e)
        })?;
        Ok(())
    }

    pub(crate) async fn touch_workout_activity_for_exercise_at<'e, E>(
        &self,
        user_id: i64,
        exercise_id: i64,
        at: DateTime<Utc>,
        executor: E,
    ) -> Result<(), AppError>
    where
        E: Executor<'e, Database = Sqlite>,
    {
        sqlx::query!(
            r#"
            UPDATE workouts
            SET last_activity_time = ?
            WHERE id = (
                SELECT workout_id
                FROM exercises
                WHERE id = ? AND user_id = ?
            )
              AND user_id = ?
            "#,
            at,
            exercise_id,
            user_id,
            user_id
        )
        .execute(executor)
        .await
        .map_err(|e| {
            error!(target: "database", "Failed to touch workout activity (exercise): {}", e);
            AppError::DatabaseError(e)
        })?;
        Ok(())
    }

    pub async fn create_workout(
        &self,
        user_id: i64,
        req: &CreateWorkoutRequest,
    ) -> Result<i64, AppError> {
        debug!(target: "database", "Creating new workout for date: {}", req.date);

        let pool = self.pool().await;
        let result = sqlx::query!(
            r#"
            INSERT INTO workouts (user_id, date, start_time, end_time, notes, timezone_offset_minutes, last_activity_time)
            VALUES (?, ?, ?, ?, ?, ?, ?)
            RETURNING id as "id!: i64"
            "#,
            user_id,
            req.date,
            req.start_time,
            req.start_time, // Initially set end_time to start_time
            req.notes,
            req.timezone_offset_minutes,
            req.start_time,
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
        user_id: i64,
        id: i64,
        end_time: DateTime<Utc>,
        notes: Option<String>,
        feedback: Option<String>,
    ) -> Result<(), AppError> {
        debug!(target: "database", "Updating workout #{} end time to {} with feedback", id, end_time);

        let pool = self.pool().await;
        let res = sqlx::query!(
            r#"
            UPDATE workouts
            SET end_time = ?, notes = ?, feedback = ?, last_activity_time = ?, auto_closed_at = NULL
            WHERE id = ? AND user_id = ?
            "#,
            end_time,
            notes,
            feedback,
            end_time,
            id,
            user_id,
        )
        .execute(&pool)
        .await
        .map_err(|e| {
            error!(target: "database", "Failed to update workout end time: {}", e);
            AppError::DatabaseError(e)
        })?;

        if res.rows_affected() == 0 {
            return Err(AppError::NotFound(format!("Workout #{} not found", id)));
        }

        info!(target: "database", "Updated workout #{} end time and feedback", id);
        Ok(())
    }

    pub async fn update_workout_times(
        &self,
        user_id: i64,
        id: i64,
        start_time: DateTime<Utc>,
        end_time: DateTime<Utc>,
        notes: Option<Option<String>>,
        feedback: Option<Option<String>>,
    ) -> Result<(), AppError> {
        debug!(
            target: "database",
            "Updating workout #{} times: start={} end={}",
            id,
            start_time,
            end_time
        );

        let now = Utc::now();
        let update_notes = notes.is_some();
        let notes_value = notes.unwrap_or(None);
        let update_feedback = feedback.is_some();
        let feedback_value = feedback.unwrap_or(None);

        let pool = self.pool().await;
        let res = sqlx::query!(
            r#"
            UPDATE workouts
            SET date = ?,
                start_time = ?,
                end_time = ?,
                last_activity_time = ?,
                auto_closed_at = NULL,
                notes = CASE WHEN ? THEN ? ELSE notes END,
                feedback = CASE WHEN ? THEN ? ELSE feedback END
            WHERE id = ? AND user_id = ?
            "#,
            start_time,
            start_time,
            end_time,
            now,
            update_notes,
            notes_value,
            update_feedback,
            feedback_value,
            id,
            user_id,
        )
        .execute(&pool)
        .await
        .map_err(|e| {
            error!(target: "database", "Failed to update workout times: {}", e);
            AppError::DatabaseError(e)
        })?;

        if res.rows_affected() == 0 {
            return Err(AppError::NotFound(format!("Workout #{} not found", id)));
        }

        info!(target: "database", "Updated workout #{} times", id);
        Ok(())
    }

    pub async fn get_workout(&self, user_id: i64, id: i64) -> Result<Workout, AppError> {
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
                feedback,
                timezone_offset_minutes,
                auto_closed_at as "auto_closed_at: DateTime<Utc>"
            FROM workouts
            WHERE id = ? AND user_id = ?
            "#,
            id,
            user_id
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
            exercise_count: None,
            timezone_offset_minutes: result.timezone_offset_minutes,
            auto_closed_at: result.auto_closed_at,
        })
    }

    pub async fn get_workouts(&self, user_id: i64) -> Result<Vec<Workout>, AppError> {
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
                feedback,
                timezone_offset_minutes,
                auto_closed_at as "auto_closed_at: DateTime<Utc>",
                (
                    SELECT COUNT(*)
                    FROM exercises e
                    WHERE e.workout_id = workouts.id
                      AND e.user_id = workouts.user_id
                ) as "exercise_count!: i64"
            FROM workouts
            WHERE user_id = ?
            ORDER BY date DESC
            "#,
            user_id
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
                exercise_count: Some(row.exercise_count),
                timezone_offset_minutes: row.timezone_offset_minutes,
                auto_closed_at: row.auto_closed_at,
            })
            .collect();

        info!(target: "database", "Retrieved {} workouts", workouts.len());
        Ok(workouts)
    }

    pub async fn delete_workout(&self, user_id: i64, id: i64) -> Result<(), AppError> {
        debug!(target: "database", "Deleting workout #{}", id);

        let pool = self.pool().await;
        // With CASCADE DELETE, we only need to delete the workout
        // and all related exercises and sets will be automatically deleted
        let res = sqlx::query!(
            r#"
            DELETE FROM workouts
            WHERE id = ? AND user_id = ?
            "#,
            id,
            user_id,
        )
        .execute(&pool)
        .await
        .map_err(|e| {
            error!(target: "database", "Failed to delete workout: {}", e);
            AppError::DatabaseError(e)
        })?;

        if res.rows_affected() == 0 {
            return Err(AppError::NotFound(format!("Workout #{} not found", id)));
        }

        info!(target: "database", "Deleted workout #{} and all its exercises and sets", id);
        Ok(())
    }

    pub async fn touch_workout_activity(
        &self,
        user_id: i64,
        workout_id: i64,
    ) -> Result<(), AppError> {
        let now = Utc::now();
        let pool = self.pool().await;
        self.touch_workout_activity_at(user_id, workout_id, now, &pool)
            .await
    }

    pub async fn touch_workout_activity_for_exercise(
        &self,
        user_id: i64,
        exercise_id: i64,
    ) -> Result<(), AppError> {
        let now = Utc::now();
        let pool = self.pool().await;
        self.touch_workout_activity_for_exercise_at(user_id, exercise_id, now, &pool)
            .await
    }

    pub async fn auto_close_stale_workouts(
        &self,
        inactivity_minutes: i64,
    ) -> Result<u64, AppError> {
        let now = Utc::now();
        let pool = self.pool().await;
        let res = sqlx::query!(
            r#"
            UPDATE workouts
            SET end_time = datetime(COALESCE(last_activity_time, start_time), '+' || ? || ' minutes'),
                auto_closed_at = ?
            WHERE auto_closed_at IS NULL
              AND end_time <= start_time
              AND (julianday(?) - julianday(COALESCE(last_activity_time, start_time))) * 24 * 60 >= ?
            "#,
            inactivity_minutes,
            now,
            now,
            inactivity_minutes
        )
        .execute(&pool)
        .await
        .map_err(|e| {
            error!(target: "database", "Failed to auto-close stale workouts: {}", e);
            AppError::DatabaseError(e)
        })?;
        Ok(res.rows_affected())
    }
}
