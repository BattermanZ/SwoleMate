use super::Database;
use crate::{errors::AppError, models::*};
use chrono::{DateTime, Utc};
use log::{debug, error, info};

impl Database {
    pub async fn create_exercise(
        &self,
        user_id: i64,
        workout_id: i64,
        req: &CreateExerciseRequest,
    ) -> Result<i64, AppError> {
        debug!(
            target: "database",
            "Creating exercise '{}' for workout #{}",
            req.exercise_type,
            workout_id
        );

        let pool = self.pool().await;
        let mut tx = pool.begin().await.map_err(AppError::DatabaseError)?;

        let workout_exists = sqlx::query_scalar!(
            r#"SELECT COUNT(*) as "count!: i64" FROM workouts WHERE id = ? AND user_id = ?"#,
            workout_id,
            user_id
        )
        .fetch_one(&mut *tx)
        .await
        .map_err(AppError::DatabaseError)?;
        if workout_exists == 0 {
            return Err(AppError::NotFound(format!(
                "Workout #{} not found",
                workout_id
            )));
        }

        let split_weight = req.split_weight.unwrap_or(false);
        let per_side_weight = req.per_side_weight.unwrap_or(false) || split_weight;

        let result = sqlx::query!(
            r#"
            INSERT INTO exercises (user_id, workout_id, exercise_type, start_time, end_time, notes, per_side_weight, split_weight)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?)
            RETURNING id as "id!: i64"
            "#,
            user_id,
            workout_id,
            req.exercise_type,
            req.start_time,
            req.start_time, // Initially set end_time to start_time
            req.notes,
            per_side_weight,
            split_weight,
        )
        .fetch_one(&mut *tx)
        .await
        .map_err(|e| {
            error!(target: "database", "Failed to create exercise: {}", e);
            AppError::DatabaseError(e)
        })?;

        let exercise_id = result.id;

        if let Some(settings) = &req.settings {
            for setting in settings {
                sqlx::query!(
                    r#"
                    INSERT INTO exercise_settings (user_id, exercise_id, setting_key, setting_value)
                    VALUES (?, ?, ?, ?)
                    "#,
                    user_id,
                    exercise_id,
                    setting.key,
                    setting.value
                )
                .execute(&mut *tx)
                .await
                .map_err(|e| {
                    error!(target: "database", "Failed to create exercise setting: {}", e);
                    AppError::DatabaseError(e)
                })?;
            }
        }

        let now = Utc::now();
        self.touch_workout_activity_at(user_id, workout_id, now, &mut *tx)
            .await?;

        tx.commit().await.map_err(AppError::DatabaseError)?;

        info!(target: "database", "Created exercise #{}", exercise_id);
        Ok(exercise_id)
    }

    pub async fn update_exercise(
        &self,
        user_id: i64,
        id: i64,
        req: &UpdateExerciseRequest,
    ) -> Result<(), AppError> {
        debug!(
            target: "database",
            "Updating exercise #{} end time to {}",
            id,
            req.end_time
        );

        let pool = self.pool().await;
        let mut tx = pool.begin().await.map_err(AppError::DatabaseError)?;

        let exists = sqlx::query_scalar!(
            r#"SELECT COUNT(*) as "count!: i64" FROM exercises WHERE id = ? AND user_id = ?"#,
            id,
            user_id
        )
        .fetch_one(&mut *tx)
        .await
        .map_err(AppError::DatabaseError)?;
        if exists == 0 {
            return Err(AppError::NotFound(format!("Exercise #{} not found", id)));
        }

        sqlx::query!(
            r#"
            UPDATE exercises
            SET end_time = ?, notes = ?
            WHERE id = ? AND user_id = ?
            "#,
            req.end_time,
            req.notes,
            id,
            user_id,
        )
        .execute(&mut *tx)
        .await
        .map_err(|e| {
            error!(target: "database", "Failed to update exercise: {}", e);
            AppError::DatabaseError(e)
        })?;

        let mut per_side_weight = req.per_side_weight;
        let mut split_weight = req.split_weight;

        // Enforce invariants:
        // - split_weight implies per_side_weight
        // - disabling per_side_weight also disables split_weight
        if per_side_weight == Some(false) {
            split_weight = Some(false);
        } else if split_weight == Some(true) {
            per_side_weight = Some(true);
        }

        if per_side_weight.is_some() || split_weight.is_some() {
            sqlx::query!(
                r#"
                UPDATE exercises
                SET per_side_weight = COALESCE(?, per_side_weight),
                    split_weight = COALESCE(?, split_weight)
                WHERE id = ? AND user_id = ?
                "#,
                per_side_weight,
                split_weight,
                id,
                user_id
            )
            .execute(&mut *tx)
            .await
            .map_err(|e| {
                error!(target: "database", "Failed to update exercise flags: {}", e);
                AppError::DatabaseError(e)
            })?;
        }

        if let Some(settings) = &req.settings {
            sqlx::query!(
                r#"
                DELETE FROM exercise_settings
                WHERE exercise_id = ? AND user_id = ?
                "#,
                id,
                user_id
            )
            .execute(&mut *tx)
            .await
            .map_err(|e| {
                error!(target: "database", "Failed to delete exercise settings: {}", e);
                AppError::DatabaseError(e)
            })?;

            for setting in settings {
                sqlx::query!(
                    r#"
                    INSERT INTO exercise_settings (user_id, exercise_id, setting_key, setting_value)
                    VALUES (?, ?, ?, ?)
                    "#,
                    user_id,
                    id,
                    setting.key,
                    setting.value
                )
                .execute(&mut *tx)
                .await
                .map_err(|e| {
                    error!(target: "database", "Failed to create exercise setting: {}", e);
                    AppError::DatabaseError(e)
                })?;
            }
        }

        let now = Utc::now();
        self.touch_workout_activity_for_exercise_at(user_id, id, now, &mut *tx)
            .await?;

        tx.commit().await.map_err(AppError::DatabaseError)?;

        info!(target: "database", "Updated exercise #{}", id);
        Ok(())
    }

    pub async fn create_set(
        &self,
        user_id: i64,
        exercise_id: i64,
        req: &CreateSetRequest,
    ) -> Result<Set, AppError> {
        debug!(
            target: "database",
            "Creating set for exercise #{}: {}x{}kg duration={:?}",
            exercise_id,
            req.reps,
            req.weight,
            req.duration_seconds
        );

        let pool = self.pool().await;
        let mut tx = pool.begin().await.map_err(AppError::DatabaseError)?;

        let exercise_exists = sqlx::query_scalar!(
            r#"SELECT COUNT(*) as "count!: i64" FROM exercises WHERE id = ? AND user_id = ?"#,
            exercise_id,
            user_id
        )
        .fetch_one(&mut *tx)
        .await
        .map_err(AppError::DatabaseError)?;
        if exercise_exists == 0 {
            return Err(AppError::NotFound(format!(
                "Exercise #{} not found",
                exercise_id
            )));
        }

        let result = sqlx::query!(
            r#"
            INSERT INTO sets (user_id, exercise_id, reps, weight, weight_left, weight_right, duration_seconds, notes)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?)
            RETURNING id as "id!: i64"
            "#,
            user_id,
            exercise_id,
            req.reps,
            req.weight,
            req.weight_left,
            req.weight_right,
            req.duration_seconds,
            req.notes,
        )
        .fetch_one(&mut *tx)
        .await
        .map_err(|e| {
            error!(target: "database", "Failed to create set: {}", e);
            AppError::DatabaseError(e)
        })?;

        let now = Utc::now();
        self.touch_workout_activity_for_exercise_at(user_id, exercise_id, now, &mut *tx)
            .await?;

        tx.commit().await.map_err(AppError::DatabaseError)?;

        debug!(target: "database", "Created set #{}", result.id);
        Ok(Set {
            id: Some(result.id),
            exercise_id,
            reps: req.reps,
            weight: req.weight,
            weight_left: req.weight_left,
            weight_right: req.weight_right,
            duration_seconds: req.duration_seconds,
            notes: req.notes.clone(),
        })
    }

    pub async fn replace_sets_for_exercise(
        &self,
        user_id: i64,
        exercise_id: i64,
        sets: &[CreateSetRequest],
    ) -> Result<Vec<Set>, AppError> {
        debug!(
            target: "database",
            "Replacing {} sets for exercise #{}",
            sets.len(),
            exercise_id
        );

        let pool = self.pool().await;
        let mut tx = pool.begin().await.map_err(AppError::DatabaseError)?;

        let exercise_exists = sqlx::query_scalar!(
            r#"SELECT COUNT(*) as "count!: i64" FROM exercises WHERE id = ? AND user_id = ?"#,
            exercise_id,
            user_id
        )
        .fetch_one(&mut *tx)
        .await
        .map_err(AppError::DatabaseError)?;
        if exercise_exists == 0 {
            return Err(AppError::NotFound(format!(
                "Exercise #{} not found",
                exercise_id
            )));
        }

        sqlx::query!(
            r#"
            DELETE FROM sets
            WHERE exercise_id = ? AND user_id = ?
            "#,
            exercise_id,
            user_id
        )
        .execute(&mut *tx)
        .await
        .map_err(|e| {
            error!(target: "database", "Failed to delete sets: {}", e);
            AppError::DatabaseError(e)
        })?;

        let mut created = Vec::with_capacity(sets.len());
        for req in sets {
            let result = sqlx::query!(
                r#"
                INSERT INTO sets (user_id, exercise_id, reps, weight, weight_left, weight_right, duration_seconds, notes)
                VALUES (?, ?, ?, ?, ?, ?, ?, ?)
                RETURNING id as "id!: i64"
                "#,
                user_id,
                exercise_id,
                req.reps,
                req.weight,
                req.weight_left,
                req.weight_right,
                req.duration_seconds,
                req.notes,
            )
            .fetch_one(&mut *tx)
            .await
            .map_err(|e| {
                error!(target: "database", "Failed to create set during replace: {}", e);
                AppError::DatabaseError(e)
            })?;

            created.push(Set {
                id: Some(result.id),
                exercise_id,
                reps: req.reps,
                weight: req.weight,
                weight_left: req.weight_left,
                weight_right: req.weight_right,
                duration_seconds: req.duration_seconds,
                notes: req.notes.clone(),
            });
        }

        let now = Utc::now();
        self.touch_workout_activity_for_exercise_at(user_id, exercise_id, now, &mut *tx)
            .await?;

        tx.commit().await.map_err(AppError::DatabaseError)?;

        info!(
            target: "database",
            "Replaced sets for exercise #{}",
            exercise_id
        );
        Ok(created)
    }

    pub async fn get_exercises_for_workout(
        &self,
        user_id: i64,
        workout_id: i64,
    ) -> Result<Vec<Exercise>, AppError> {
        debug!(target: "database", "Fetching exercises for workout #{}", workout_id);

        let pool = self.pool().await;
        let rows = sqlx::query!(
            r#"
            SELECT 
                id as "id?",
                workout_id,
                exercise_type,
                start_time as "start_time: DateTime<Utc>",
                end_time as "end_time: DateTime<Utc>",
                notes,
                per_side_weight as "per_side_weight: bool",
                split_weight as "split_weight: bool"
            FROM exercises
            WHERE workout_id = ? AND user_id = ?
            "#,
            workout_id,
            user_id
        )
        .fetch_all(&pool)
        .await
        .map_err(|e| {
            error!(target: "database", "Failed to fetch exercises: {}", e);
            AppError::DatabaseError(e)
        })?;

        let mut exercises = Vec::with_capacity(rows.len());
        for row in rows {
            let id = row
                .id
                .ok_or_else(|| AppError::InternalError("Exercise row missing id".to_string()))?;
            let settings = self.get_settings_for_exercise(user_id, id).await?;
            exercises.push(Exercise {
                id: Some(id),
                workout_id: row.workout_id,
                exercise_type: row.exercise_type,
                start_time: row.start_time,
                end_time: row.end_time,
                notes: row.notes,
                per_side_weight: row.per_side_weight,
                split_weight: row.split_weight,
                settings,
            });
        }

        debug!(target: "database", "Found {} exercises for workout #{}", exercises.len(), workout_id);
        Ok(exercises)
    }

    pub async fn get_sets_for_exercise(
        &self,
        user_id: i64,
        exercise_id: i64,
    ) -> Result<Vec<Set>, AppError> {
        debug!(target: "database", "Fetching sets for exercise #{}", exercise_id);

        let pool = self.pool().await;
        let sets = sqlx::query_as!(
            Set,
            r#"
            SELECT id as "id?", exercise_id, reps, weight, weight_left, weight_right, duration_seconds, notes
            FROM sets
            WHERE exercise_id = ? AND user_id = ?
            "#,
            exercise_id,
            user_id
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

    pub async fn get_unique_exercise_types(&self, user_id: i64) -> Result<Vec<String>, AppError> {
        debug!(target: "database", "Fetching unique exercise types");

        let pool = self.pool().await;
        let rows = sqlx::query!(
            r#"
            SELECT DISTINCT exercise_type
            FROM exercises
            WHERE user_id = ?
            ORDER BY exercise_type ASC
            "#,
            user_id
        )
        .fetch_all(&pool)
        .await
        .map_err(|e| {
            error!(target: "database", "Failed to fetch exercise types: {}", e);
            AppError::DatabaseError(e)
        })?;

        let row_count = rows.len();
        let exercise_types = rows.into_iter().map(|row| row.exercise_type).collect();

        debug!(target: "database", "Found {} unique exercise types", row_count);
        Ok(exercise_types)
    }

    pub async fn get_last_exercise_data(
        &self,
        user_id: i64,
        exercise_type: &str,
        exclude_workout_id: Option<i64>,
    ) -> Result<Option<(Exercise, Vec<Set>)>, AppError> {
        debug!(target: "database", "Fetching last data for exercise type: {}", exercise_type);

        let pool = self.pool().await;
        let row = sqlx::query!(
            r#"
            SELECT
                id as "id?",
                workout_id,
                exercise_type,
                start_time as "start_time: DateTime<Utc>",
                end_time as "end_time: DateTime<Utc>",
                notes,
                per_side_weight as "per_side_weight: bool",
                split_weight as "split_weight: bool"
            FROM exercises
            WHERE user_id = ?
              AND LOWER(exercise_type) = LOWER(?)
              AND (?3 IS NULL OR workout_id != ?3)
            ORDER BY start_time DESC
            LIMIT 1
            "#,
            user_id,
            exercise_type,
            exclude_workout_id
        )
        .fetch_optional(&pool)
        .await
        .map_err(|e| {
            error!(target: "database", "Failed to fetch last exercise data: {}", e);
            AppError::DatabaseError(e)
        })?;

        if let Some(row) = row {
            let exercise_id = row.id.ok_or_else(|| {
                AppError::InternalError("Exercise row missing id for sets lookup".to_string())
            })?;
            let sets = self.get_sets_for_exercise(user_id, exercise_id).await?;

            let mut exercise = Exercise {
                id: row.id,
                workout_id: row.workout_id,
                exercise_type: row.exercise_type,
                start_time: row.start_time,
                end_time: row.end_time,
                notes: row.notes,
                per_side_weight: row.per_side_weight,
                split_weight: row.split_weight,
                settings: Vec::new(),
            };

            exercise.settings = self.get_settings_for_exercise(user_id, exercise_id).await?;
            Ok(Some((exercise, sets)))
        } else {
            Ok(None)
        }
    }

    pub async fn get_settings_for_exercise(
        &self,
        user_id: i64,
        exercise_id: i64,
    ) -> Result<Vec<ExerciseSetting>, AppError> {
        let pool = self.pool().await;
        let rows = sqlx::query_as!(
            ExerciseSetting,
            r#"
            SELECT id as "id?", exercise_id, setting_key, setting_value
            FROM exercise_settings
            WHERE exercise_id = ? AND user_id = ?
            ORDER BY id ASC
            "#,
            exercise_id,
            user_id
        )
        .fetch_all(&pool)
        .await
        .map_err(|e| {
            error!(target: "database", "Failed to fetch exercise settings: {}", e);
            AppError::DatabaseError(e)
        })?;

        Ok(rows)
    }

    pub async fn delete_exercise(&self, user_id: i64, id: i64) -> Result<(), AppError> {
        debug!(target: "database", "Deleting exercise #{}", id);

        let pool = self.pool().await;
        // With CASCADE DELETE, we only need to delete the exercise
        // and all related sets will be automatically deleted
        let res = sqlx::query!(
            r#"
            DELETE FROM exercises
            WHERE id = ? AND user_id = ?
            "#,
            id,
            user_id,
        )
        .execute(&pool)
        .await
        .map_err(|e| {
            error!(target: "database", "Failed to delete exercise: {}", e);
            AppError::DatabaseError(e)
        })?;

        if res.rows_affected() == 0 {
            return Err(AppError::NotFound(format!("Exercise #{} not found", id)));
        }

        info!(target: "database", "Deleted exercise #{} and its sets", id);
        Ok(())
    }
}
