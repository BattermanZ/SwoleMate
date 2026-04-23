use super::Database;
use crate::{errors::AppError, models::*};
use chrono::Utc;
use log::{debug, error, info};
use sqlx::{Row, Sqlite, Transaction};

impl Database {
    async fn get_template_settings_for_exercise(
        &self,
        user_id: i64,
        template_exercise_id: i64,
        pool: &sqlx::Pool<Sqlite>,
    ) -> Result<Vec<WorkoutTemplateExerciseSetting>, AppError> {
        let rows = sqlx::query(
            r#"
            SELECT id, template_exercise_id, setting_key, setting_value
            FROM workout_template_exercise_settings
            WHERE template_exercise_id = ? AND user_id = ?
            ORDER BY id ASC
            "#,
        )
        .bind(template_exercise_id)
        .bind(user_id)
        .fetch_all(pool)
        .await
        .map_err(|e| {
            error!(target: "database", "Failed to fetch template exercise settings: {}", e);
            AppError::DatabaseError(e)
        })?;

        Ok(rows
            .into_iter()
            .map(|row| WorkoutTemplateExerciseSetting {
                id: row.get("id"),
                template_exercise_id: row.get("template_exercise_id"),
                setting_key: row.get("setting_key"),
                setting_value: row.get("setting_value"),
            })
            .collect())
    }

    async fn get_template_exercises(
        &self,
        user_id: i64,
        template_id: i64,
        pool: &sqlx::Pool<Sqlite>,
    ) -> Result<Vec<WorkoutTemplateExercise>, AppError> {
        let rows = sqlx::query(
            r#"
            SELECT id, template_id, position, exercise_type, notes, per_side_weight, split_weight
            FROM workout_template_exercises
            WHERE template_id = ? AND user_id = ?
            ORDER BY position ASC, id ASC
            "#,
        )
        .bind(template_id)
        .bind(user_id)
        .fetch_all(pool)
        .await
        .map_err(|e| {
            error!(target: "database", "Failed to fetch template exercises: {}", e);
            AppError::DatabaseError(e)
        })?;

        let mut exercises = Vec::with_capacity(rows.len());
        for row in rows {
            let id: i64 = row.get("id");
            let settings = self
                .get_template_settings_for_exercise(user_id, id, pool)
                .await?;
            exercises.push(WorkoutTemplateExercise {
                id,
                template_id: row.get("template_id"),
                position: row.get("position"),
                exercise_type: row.get("exercise_type"),
                notes: row.get("notes"),
                per_side_weight: row.get::<i64, _>("per_side_weight") != 0,
                split_weight: row.get::<i64, _>("split_weight") != 0,
                settings,
            });
        }

        Ok(exercises)
    }

    async fn insert_template_exercises(
        &self,
        user_id: i64,
        template_id: i64,
        exercises: &[WorkoutTemplateExerciseRequest],
        tx: &mut Transaction<'_, Sqlite>,
    ) -> Result<(), AppError> {
        for (index, exercise) in exercises.iter().enumerate() {
            let split_weight = exercise.split_weight.unwrap_or(false);
            let per_side_weight = exercise.per_side_weight.unwrap_or(false) || split_weight;

            let result = sqlx::query(
                r#"
                INSERT INTO workout_template_exercises (
                    user_id,
                    template_id,
                    position,
                    exercise_type,
                    notes,
                    per_side_weight,
                    split_weight
                )
                VALUES (?, ?, ?, ?, ?, ?, ?)
                RETURNING id
                "#,
            )
            .bind(user_id)
            .bind(template_id)
            .bind(index as i64)
            .bind(&exercise.exercise_type)
            .bind(&exercise.notes)
            .bind(per_side_weight)
            .bind(split_weight)
            .fetch_one(&mut **tx)
            .await
            .map_err(|e| {
                error!(target: "database", "Failed to insert template exercise: {}", e);
                AppError::DatabaseError(e)
            })?;

            let template_exercise_id: i64 = result.get("id");
            if let Some(settings) = exercise.settings.as_ref() {
                for setting in settings {
                    sqlx::query(
                        r#"
                        INSERT INTO workout_template_exercise_settings (
                            user_id,
                            template_exercise_id,
                            setting_key,
                            setting_value
                        )
                        VALUES (?, ?, ?, ?)
                        "#,
                    )
                    .bind(user_id)
                    .bind(template_exercise_id)
                    .bind(&setting.key)
                    .bind(&setting.value)
                    .execute(&mut **tx)
                    .await
                    .map_err(|e| {
                        error!(target: "database", "Failed to insert template setting: {}", e);
                        AppError::DatabaseError(e)
                    })?;
                }
            }
        }

        Ok(())
    }

    pub async fn list_workout_templates(
        &self,
        user_id: i64,
    ) -> Result<Vec<WorkoutTemplate>, AppError> {
        debug!(target: "database", "Listing workout templates");

        let pool = self.pool().await;
        let rows = sqlx::query(
            r#"
            SELECT
                t.id,
                t.name,
                t.created_at,
                t.updated_at,
                COUNT(e.id) AS exercise_count
            FROM workout_templates t
            LEFT JOIN workout_template_exercises e
                ON e.template_id = t.id AND e.user_id = t.user_id
            WHERE t.user_id = ?
            GROUP BY t.id, t.name, t.created_at, t.updated_at
            ORDER BY t.updated_at DESC, t.id DESC
            "#,
        )
        .bind(user_id)
        .fetch_all(&pool)
        .await
        .map_err(|e| {
            error!(target: "database", "Failed to list templates: {}", e);
            AppError::DatabaseError(e)
        })?;

        Ok(rows
            .into_iter()
            .map(|row| WorkoutTemplate {
                id: row.get("id"),
                name: row.get("name"),
                exercise_count: row.get("exercise_count"),
                created_at: row.get("created_at"),
                updated_at: row.get("updated_at"),
            })
            .collect())
    }

    pub async fn get_workout_template(
        &self,
        user_id: i64,
        template_id: i64,
    ) -> Result<WorkoutTemplateDetail, AppError> {
        debug!(target: "database", "Fetching workout template #{}", template_id);

        let pool = self.pool().await;
        let row = sqlx::query(
            r#"
            SELECT
                t.id,
                t.name,
                t.created_at,
                t.updated_at,
                COUNT(e.id) AS exercise_count
            FROM workout_templates t
            LEFT JOIN workout_template_exercises e
                ON e.template_id = t.id AND e.user_id = t.user_id
            WHERE t.id = ? AND t.user_id = ?
            GROUP BY t.id, t.name, t.created_at, t.updated_at
            "#,
        )
        .bind(template_id)
        .bind(user_id)
        .fetch_optional(&pool)
        .await
        .map_err(|e| {
            error!(target: "database", "Failed to fetch template: {}", e);
            AppError::DatabaseError(e)
        })?
        .ok_or_else(|| AppError::NotFound(format!("Template #{} not found", template_id)))?;

        let template = WorkoutTemplate {
            id: row.get("id"),
            name: row.get("name"),
            exercise_count: row.get("exercise_count"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
        };
        let exercises = self
            .get_template_exercises(user_id, template_id, &pool)
            .await?;

        Ok(WorkoutTemplateDetail {
            template,
            exercises,
        })
    }

    pub async fn create_workout_template(
        &self,
        user_id: i64,
        req: &CreateWorkoutTemplateRequest,
    ) -> Result<WorkoutTemplateDetail, AppError> {
        debug!(target: "database", "Creating workout template '{}'", req.name);

        let pool = self.pool().await;
        let mut tx = pool.begin().await.map_err(AppError::DatabaseError)?;
        let now = Utc::now();

        let result = sqlx::query(
            r#"
            INSERT INTO workout_templates (user_id, name, created_at, updated_at)
            VALUES (?, ?, ?, ?)
            RETURNING id
            "#,
        )
        .bind(user_id)
        .bind(&req.name)
        .bind(now)
        .bind(now)
        .fetch_one(&mut *tx)
        .await
        .map_err(|e| {
            error!(target: "database", "Failed to create template: {}", e);
            AppError::DatabaseError(e)
        })?;

        let template_id: i64 = result.get("id");
        self.insert_template_exercises(user_id, template_id, &req.exercises, &mut tx)
            .await?;

        tx.commit().await.map_err(AppError::DatabaseError)?;
        info!(target: "database", "Created template #{}", template_id);
        self.get_workout_template(user_id, template_id).await
    }

    pub async fn update_workout_template(
        &self,
        user_id: i64,
        template_id: i64,
        req: &UpdateWorkoutTemplateRequest,
    ) -> Result<WorkoutTemplateDetail, AppError> {
        debug!(target: "database", "Updating workout template #{}", template_id);

        let pool = self.pool().await;
        let mut tx = pool.begin().await.map_err(AppError::DatabaseError)?;
        let now = Utc::now();

        let result = sqlx::query(
            r#"
            UPDATE workout_templates
            SET name = ?, updated_at = ?
            WHERE id = ? AND user_id = ?
            "#,
        )
        .bind(&req.name)
        .bind(now)
        .bind(template_id)
        .bind(user_id)
        .execute(&mut *tx)
        .await
        .map_err(|e| {
            error!(target: "database", "Failed to update template: {}", e);
            AppError::DatabaseError(e)
        })?;

        if result.rows_affected() == 0 {
            return Err(AppError::NotFound(format!(
                "Template #{} not found",
                template_id
            )));
        }

        sqlx::query(
            r#"
            DELETE FROM workout_template_exercises
            WHERE template_id = ? AND user_id = ?
            "#,
        )
        .bind(template_id)
        .bind(user_id)
        .execute(&mut *tx)
        .await
        .map_err(|e| {
            error!(target: "database", "Failed to replace template exercises: {}", e);
            AppError::DatabaseError(e)
        })?;

        self.insert_template_exercises(user_id, template_id, &req.exercises, &mut tx)
            .await?;

        tx.commit().await.map_err(AppError::DatabaseError)?;
        info!(target: "database", "Updated template #{}", template_id);
        self.get_workout_template(user_id, template_id).await
    }

    pub async fn delete_workout_template(
        &self,
        user_id: i64,
        template_id: i64,
    ) -> Result<(), AppError> {
        debug!(target: "database", "Deleting workout template #{}", template_id);

        let pool = self.pool().await;
        let result = sqlx::query(
            r#"
            DELETE FROM workout_templates
            WHERE id = ? AND user_id = ?
            "#,
        )
        .bind(template_id)
        .bind(user_id)
        .execute(&pool)
        .await
        .map_err(|e| {
            error!(target: "database", "Failed to delete template: {}", e);
            AppError::DatabaseError(e)
        })?;

        if result.rows_affected() == 0 {
            return Err(AppError::NotFound(format!(
                "Template #{} not found",
                template_id
            )));
        }

        info!(target: "database", "Deleted template #{}", template_id);
        Ok(())
    }

    pub async fn start_workout_from_template(
        &self,
        user_id: i64,
        template_id: i64,
        req: &StartWorkoutFromTemplateRequest,
    ) -> Result<i64, AppError> {
        debug!(
            target: "database",
            "Starting workout from template #{}",
            template_id
        );

        let template = self.get_workout_template(user_id, template_id).await?;
        let pool = self.pool().await;
        let mut tx = pool.begin().await.map_err(AppError::DatabaseError)?;

        let workout = CreateWorkoutRequest {
            date: req.date,
            start_time: req.start_time,
            notes: None,
            timezone_offset_minutes: req.timezone_offset_minutes,
        };
        workout.validate().map_err(AppError::BadRequest)?;

        let result = sqlx::query(
            r#"
            INSERT INTO workouts (
                user_id,
                date,
                start_time,
                end_time,
                notes,
                timezone_offset_minutes,
                last_activity_time
            )
            VALUES (?, ?, ?, ?, ?, ?, ?)
            RETURNING id
            "#,
        )
        .bind(user_id)
        .bind(workout.date)
        .bind(workout.start_time)
        .bind(workout.start_time)
        .bind(workout.notes)
        .bind(workout.timezone_offset_minutes)
        .bind(workout.start_time)
        .fetch_one(&mut *tx)
        .await
        .map_err(|e| {
            error!(
                target: "database",
                "Failed to create workout from template #{}: {}",
                template_id,
                e
            );
            AppError::DatabaseError(e)
        })?;

        let workout_id: i64 = result.get("id");

        for exercise in template.exercises {
            let exercise_req = CreateExerciseRequest {
                exercise_type: exercise.exercise_type,
                start_time: req.start_time,
                notes: exercise.notes,
                per_side_weight: Some(exercise.per_side_weight),
                split_weight: Some(exercise.split_weight),
                settings: Some(
                    exercise
                        .settings
                        .into_iter()
                        .map(|setting| ExerciseSettingRequest {
                            key: setting.setting_key,
                            value: setting.setting_value,
                        })
                        .collect(),
                ),
            };
            exercise_req.validate().map_err(AppError::BadRequest)?;

            let split_weight = exercise_req.split_weight.unwrap_or(false);
            let per_side_weight = exercise_req.per_side_weight.unwrap_or(false) || split_weight;

            let result = sqlx::query(
                r#"
                INSERT INTO exercises (
                    user_id,
                    workout_id,
                    exercise_type,
                    start_time,
                    end_time,
                    notes,
                    per_side_weight,
                    split_weight
                )
                VALUES (?, ?, ?, ?, ?, ?, ?, ?)
                RETURNING id
                "#,
            )
            .bind(user_id)
            .bind(workout_id)
            .bind(&exercise_req.exercise_type)
            .bind(exercise_req.start_time)
            .bind(exercise_req.start_time)
            .bind(&exercise_req.notes)
            .bind(per_side_weight)
            .bind(split_weight)
            .fetch_one(&mut *tx)
            .await
            .map_err(|e| {
                error!(
                    target: "database",
                    "Failed to create template exercise for workout #{}: {}",
                    workout_id,
                    e
                );
                AppError::DatabaseError(e)
            })?;

            let exercise_id: i64 = result.get("id");
            if let Some(settings) = exercise_req.settings.as_ref() {
                for setting in settings {
                    sqlx::query(
                        r#"
                        INSERT INTO exercise_settings (user_id, exercise_id, setting_key, setting_value)
                        VALUES (?, ?, ?, ?)
                        "#,
                    )
                    .bind(user_id)
                    .bind(exercise_id)
                    .bind(&setting.key)
                    .bind(&setting.value)
                    .execute(&mut *tx)
                    .await
                    .map_err(|e| {
                        error!(
                            target: "database",
                            "Failed to create settings for template exercise #{}: {}",
                            exercise_id,
                            e
                        );
                        AppError::DatabaseError(e)
                    })?;
                }
            }
        }

        tx.commit().await.map_err(AppError::DatabaseError)?;
        info!(
            target: "database",
            "Started workout #{} from template #{}",
            workout_id,
            template_id
        );
        Ok(workout_id)
    }
}
