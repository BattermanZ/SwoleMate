use super::Database;
use crate::{errors::AppError, models::*};
use chrono::{DateTime, Utc};
use log::{debug, error};
use serde_json::json;
use sqlx::Row;

impl Database {
    pub async fn get_exercise_progress(
        &self,
        exercise_type: &str,
    ) -> Result<Vec<(Exercise, Vec<Set>)>, AppError> {
        debug!(target: "database", "Fetching progress data for exercise type: {}", exercise_type);

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
        for row in rows {
            let exercise_id = row.id.ok_or_else(|| {
                AppError::InternalError(
                    "Exercise row missing id for progress sets lookup".to_string(),
                )
            })?;
            let sets = self.get_sets_for_exercise(exercise_id).await?;

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

            exercise.settings = self.get_settings_for_exercise(exercise_id).await?;
            result.push((exercise, sets));
        }

        debug!(target: "database", "Found {} exercises for progress data", result.len());
        Ok(result)
    }

    pub async fn get_workout_stats(&self) -> Result<serde_json::Value, AppError> {
        debug!(target: "database", "Calculating workout statistics");

        let pool = self.pool().await;
        let stats = sqlx::query!(
            r#"
            WITH workout_times AS (
                SELECT 
                    strftime('%H', start_time) as hour,
                    ROUND((julianday(end_time) - julianday(start_time)) * 24 * 60) as duration,
                    date(start_time) as workout_date,
                    strftime('%Y-%W', start_time) as week,
                    feedback
                FROM workouts
            ),
            feedback_counts AS (
                SELECT 
                    COUNT(*) FILTER (WHERE feedback = '😊') as good_workouts,
                    COUNT(*) FILTER (WHERE feedback = '😐') as neutral_workouts,
                    COUNT(*) FILTER (WHERE feedback = '😞') as bad_workouts
                FROM workout_times
            ),
            popular_hours AS (
                SELECT hour, COUNT(*) as count
                FROM workout_times
                GROUP BY hour
                ORDER BY count DESC
                LIMIT 3
            ),
            weekly_counts AS (
                SELECT week, COUNT(*) as workouts_per_week
                FROM workout_times
                GROUP BY week
            ),
            avg_weekly AS (
                SELECT ROUND(AVG(workouts_per_week), 1) as avg_workouts_per_week
                FROM weekly_counts
            ),
            last_four_complete_weeks AS (
                SELECT DISTINCT week
                FROM workout_times
                WHERE week < strftime('%Y-%W', 'now')
                ORDER BY week DESC
                LIMIT 4
            ),
            recent_weekly_avg AS (
                SELECT COALESCE(
                    ROUND(CAST(COUNT(*) AS FLOAT) / 
                        NULLIF((
                            SELECT COUNT(DISTINCT week) 
                            FROM workout_times 
                            WHERE week IN (SELECT week FROM last_four_complete_weeks)
                        ), 0), 
                    1),
                    0
                ) as recent_avg
                FROM workout_times
                WHERE week IN (SELECT week FROM last_four_complete_weeks)
            ),
            recent_duration_avg AS (
                SELECT COALESCE(
                    ROUND(AVG(duration), 1),
                    0
                ) as recent_avg
                FROM workout_times
                WHERE week IN (SELECT week FROM last_four_complete_weeks)
            ),
            duration_ranges AS (
                SELECT 
                    CASE 
                        WHEN duration < 30 THEN '0-30'
                        WHEN duration < 60 THEN '30-60'
                        WHEN duration < 90 THEN '60-90'
                        ELSE '90+'
                    END as duration_range,
                    COUNT(*) as count
                FROM workout_times
                GROUP BY 
                    CASE 
                        WHEN duration < 30 THEN '0-30'
                        WHEN duration < 60 THEN '30-60'
                        WHEN duration < 90 THEN '60-90'
                        ELSE '90+'
                    END
            )
            SELECT 
                COUNT(*) as total_workouts,
                COALESCE(ROUND(AVG(duration), 1), 0) as avg_duration,
                (SELECT good_workouts FROM feedback_counts) as good_workouts,
                (SELECT neutral_workouts FROM feedback_counts) as neutral_workouts,
                (SELECT bad_workouts FROM feedback_counts) as bad_workouts,
                COALESCE((SELECT GROUP_CONCAT(hour || ':' || count) FROM popular_hours), '') as popular_hours,
                COALESCE((SELECT avg_workouts_per_week FROM avg_weekly), 0) as avg_workouts_per_week,
                COALESCE((SELECT GROUP_CONCAT(duration_range || ':' || count) FROM duration_ranges), '') as duration_distribution,
                COALESCE(
                    (
                        SELECT ROUND(
                            recent_avg - avg_workouts_per_week,
                            1
                        )
                        FROM recent_weekly_avg, avg_weekly
                        WHERE recent_avg IS NOT NULL
                    ),
                    0
                ) as frequency_trend,
                COALESCE(
                    (
                        SELECT ROUND(
                            recent_avg - AVG(duration),
                            1
                        )
                        FROM recent_duration_avg, workout_times
                        WHERE recent_avg IS NOT NULL
                    ),
                    0
                ) as duration_trend
            FROM workout_times
            "#
        )
        .fetch_one(&pool)
        .await
        .map_err(|e| {
            error!(target: "database", "Failed to fetch workout stats: {}", e);
            AppError::DatabaseError(e)
        })?;

        let popular_hours_raw = stats.popular_hours.unwrap_or_default();
        let popular_hours = if popular_hours_raw.is_empty() {
            None
        } else {
            Some(
                popular_hours_raw
                    .split(',')
                    .filter_map(|pair| {
                        let parts: Vec<&str> = pair.split(':').collect();
                        if parts.len() == 2 {
                            Some(json!({
                                "hour": parts[0],
                                "count": parts[1].parse::<i64>().unwrap_or(0)
                            }))
                        } else {
                            None
                        }
                    })
                    .collect::<Vec<_>>(),
            )
        };

        let duration_distribution_raw = stats.duration_distribution.unwrap_or_default();
        let duration_distribution = if duration_distribution_raw.is_empty() {
            None
        } else {
            Some(
                duration_distribution_raw
                    .split(',')
                    .filter_map(|pair| {
                        let parts: Vec<&str> = pair.split(':').collect();
                        if parts.len() == 2 {
                            Some(json!({
                                "range": parts[0],
                                "count": parts[1].parse::<i64>().unwrap_or(0)
                            }))
                        } else {
                            None
                        }
                    })
                    .collect::<Vec<_>>(),
            )
        };

        let monthly_rows = sqlx::query(
            r#"
            WITH RECURSIVE months(month_start) AS (
                SELECT date('now', 'start of month', '-11 months')
                UNION ALL
                SELECT date(month_start, '+1 month')
                FROM months
                WHERE month_start < date('now', 'start of month')
            ),
            counts AS (
                SELECT
                    strftime('%Y-%m', start_time) as month,
                    COUNT(*) as count
                FROM workouts
                WHERE date(start_time) >= date('now', 'start of month', '-11 months')
                GROUP BY strftime('%Y-%m', start_time)
            )
            SELECT
                strftime('%Y-%m', months.month_start) as month,
                COALESCE(counts.count, 0) as count
            FROM months
            LEFT JOIN counts ON counts.month = strftime('%Y-%m', months.month_start)
            ORDER BY months.month_start ASC
            "#,
        )
        .fetch_all(&pool)
        .await
        .map_err(|e| {
            error!(target: "database", "Failed to fetch rolling-year monthly sessions: {}", e);
            AppError::DatabaseError(e)
        })?;

        let monthly_sessions = monthly_rows
            .into_iter()
            .filter_map(|row| {
                let month: Result<String, _> = row.try_get("month");
                let count: Result<i64, _> = row.try_get("count");
                match (month, count) {
                    (Ok(month), Ok(count)) => Some(json!({ "month": month, "count": count })),
                    _ => None,
                }
            })
            .collect::<Vec<_>>();

        let avg_exercise_rows = sqlx::query!(
            r#"
            WITH recent AS (
                SELECT
                    workouts.start_time as start_time,
                    workouts.end_time as end_time,
                    CAST(ROUND((julianday(workouts.end_time) - julianday(workouts.start_time)) * 24 * 60) AS INTEGER) as duration_minutes,
                    (
                        SELECT COUNT(*)
                        FROM exercises e
                        WHERE e.workout_id = workouts.id
                    ) as exercise_count
                FROM workouts
                WHERE end_time > start_time
                ORDER BY start_time DESC
                LIMIT 60
            )
            SELECT
                start_time as "start_time: DateTime<Utc>",
                end_time as "end_time: DateTime<Utc>",
                duration_minutes as "duration_minutes!: i64",
                exercise_count as "exercise_count!: i64",
                ROUND(CAST(duration_minutes AS FLOAT) / exercise_count, 2) as "avg_minutes!: f64"
            FROM recent
            WHERE exercise_count > 0
            ORDER BY start_time ASC
            "#
        )
        .fetch_all(&pool)
        .await
        .map_err(|e| {
            error!(
                target: "database",
                "Failed to fetch avg exercise duration series: {}",
                e
            );
            AppError::DatabaseError(e)
        })?;

        let avg_exercise_duration_series = avg_exercise_rows
            .into_iter()
            .filter(|row| row.duration_minutes > 0)
            .map(|row| {
                json!({
                    "start_time": row.start_time,
                    "end_time": row.end_time,
                    "duration_minutes": row.duration_minutes,
                    "exercise_count": row.exercise_count,
                    "avg_minutes": row.avg_minutes
                })
            })
            .collect::<Vec<_>>();

        let session_start_rows = sqlx::query!(
            r#"
            SELECT
                start_time as "start_time: DateTime<Utc>",
                timezone_offset_minutes
            FROM workouts
            WHERE end_time > start_time
              AND date(start_time) >= date('now', 'start of month', '-11 months')
            ORDER BY start_time ASC
            "#
        )
        .fetch_all(&pool)
        .await
        .map_err(|e| {
            error!(
                target: "database",
                "Failed to fetch rolling-year session start times: {}",
                e
            );
            AppError::DatabaseError(e)
        })?;

        let session_start_times = session_start_rows
            .iter()
            .map(|row| row.start_time)
            .collect::<Vec<_>>();

        let session_start_samples = session_start_rows
            .into_iter()
            .map(|row| {
                json!({
                    "start_time": row.start_time,
                    "timezone_offset_minutes": row.timezone_offset_minutes
                })
            })
            .collect::<Vec<_>>();

        Ok(json!({
            "total_workouts": stats.total_workouts,
            "average_duration_minutes": stats.avg_duration,
            "feedback_distribution": {
                "good": stats.good_workouts,
                "neutral": stats.neutral_workouts,
                "bad": stats.bad_workouts
            },
            "workout_frequency": {
                "average_per_week": stats.avg_workouts_per_week,
                "trend": stats.frequency_trend
            },
            "duration_trend": stats.duration_trend,
            "popular_hours": popular_hours,
            "duration_distribution": duration_distribution,
            "sessions_per_month": monthly_sessions,
            "avg_exercise_duration_series": avg_exercise_duration_series,
            "session_start_times": session_start_times,
            "session_start_samples": session_start_samples
        }))
    }

    pub async fn get_volume_stats(
        &self,
        exercise_type: &str,
    ) -> Result<serde_json::Value, AppError> {
        debug!(target: "database", "Calculating volume statistics for {}", exercise_type);

        let pool = self.pool().await;
        let weekly_volume = sqlx::query!(
            r#"
            WITH exercise_stats AS (
                SELECT 
                    e.start_time,
                    s.reps,
                    CASE
                        WHEN e.per_side_weight = 1 THEN
                            CASE
                                WHEN e.split_weight = 1 AND s.weight_left IS NOT NULL AND s.weight_right IS NOT NULL
                                    THEN (s.weight_left + s.weight_right)
                                ELSE (s.weight * 2)
                            END
                        ELSE s.weight
                    END as weight,
                    s.reps * (
                        CASE
                            WHEN e.per_side_weight = 1 THEN
                                CASE
                                    WHEN e.split_weight = 1 AND s.weight_left IS NOT NULL AND s.weight_right IS NOT NULL
                                        THEN (s.weight_left + s.weight_right)
                                    ELSE (s.weight * 2)
                                END
                            ELSE s.weight
                        END
                    ) as volume,
                    ROUND(
                        (
                            CASE
                                WHEN e.per_side_weight = 1 THEN
                                    CASE
                                        WHEN e.split_weight = 1 AND s.weight_left IS NOT NULL AND s.weight_right IS NOT NULL
                                            THEN (s.weight_left + s.weight_right)
                                        ELSE (s.weight * 2)
                                    END
                                ELSE s.weight
                            END
                        ) * (36.0 / (37.0 - CAST(s.reps AS FLOAT))),
                        2
                    ) as estimated_1rm
                FROM exercises e
                JOIN sets s ON e.id = s.exercise_id
                WHERE LOWER(e.exercise_type) = LOWER(?)
            ),
            weekly_stats AS (
                SELECT 
                    strftime('%Y-%W', start_time) as week,
                    SUM(volume) as total_volume,
                    MAX(weight) as max_weight,
                    SUM(reps) as total_reps,
                    COUNT(*) as total_sets,
                    MAX(estimated_1rm) as max_estimated_1rm,
                    GROUP_CONCAT(CAST(reps AS TEXT) || 'x' || CAST(ROUND(weight, 1) AS TEXT)) as set_schemes
                FROM exercise_stats
                GROUP BY strftime('%Y-%W', start_time)
            )
            SELECT 
                week as "week!: String",
                COALESCE(total_volume, 0.0) as "total_volume!: f64",
                COALESCE(max_weight, 0.0) as "max_weight!: f64",
                COALESCE(total_reps, 0) as "total_reps!: i64",
                total_sets as "total_sets!: i64",
                COALESCE(max_estimated_1rm, 0.0) as "max_estimated_1rm!: f64",
                COALESCE(set_schemes, '') as "set_schemes!: String"
            FROM weekly_stats
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

        let monthly_volume = sqlx::query!(
            r#"
            SELECT 
                strftime('%Y-%m', e.start_time) as month,
                SUM(
                    s.reps * (
                        CASE
                            WHEN e.per_side_weight = 1 THEN
                                CASE
                                    WHEN e.split_weight = 1 AND s.weight_left IS NOT NULL AND s.weight_right IS NOT NULL
                                        THEN (s.weight_left + s.weight_right)
                                    ELSE (s.weight * 2)
                                END
                            ELSE s.weight
                        END
                    )
                ) as total_volume,
                MAX(
                    CASE
                        WHEN e.per_side_weight = 1 THEN
                            CASE
                                WHEN e.split_weight = 1 AND s.weight_left IS NOT NULL AND s.weight_right IS NOT NULL
                                    THEN (s.weight_left + s.weight_right)
                                ELSE (s.weight * 2)
                            END
                        ELSE s.weight
                    END
                ) as max_weight,
                SUM(s.reps) as total_reps,
                COUNT(*) as total_sets
            FROM exercises e
            JOIN sets s ON e.id = s.exercise_id
            WHERE LOWER(e.exercise_type) = LOWER(?)
            GROUP BY strftime('%Y-%m', e.start_time)
            ORDER BY month ASC
            "#,
            exercise_type
        )
        .fetch_all(&pool)
        .await
        .map_err(|e| {
            error!(target: "database", "Failed to fetch monthly volume: {}", e);
            AppError::DatabaseError(e)
        })?;

        let personal_records = sqlx::query!(
            r#"
            WITH exercise_stats AS (
                SELECT 
                    s.reps,
                    CASE
                        WHEN e.per_side_weight = 1 THEN
                            CASE
                                WHEN e.split_weight = 1 AND s.weight_left IS NOT NULL AND s.weight_right IS NOT NULL
                                    THEN (s.weight_left + s.weight_right)
                                ELSE (s.weight * 2)
                            END
                        ELSE s.weight
                    END as weight,
                    s.reps * (
                        CASE
                            WHEN e.per_side_weight = 1 THEN
                                CASE
                                    WHEN e.split_weight = 1 AND s.weight_left IS NOT NULL AND s.weight_right IS NOT NULL
                                        THEN (s.weight_left + s.weight_right)
                                    ELSE (s.weight * 2)
                                END
                            ELSE s.weight
                        END
                    ) as volume,
                    ROUND(
                        (
                            CASE
                                WHEN e.per_side_weight = 1 THEN
                                    CASE
                                        WHEN e.split_weight = 1 AND s.weight_left IS NOT NULL AND s.weight_right IS NOT NULL
                                            THEN (s.weight_left + s.weight_right)
                                        ELSE (s.weight * 2)
                                    END
                                ELSE s.weight
                            END
                        ) * (36.0 / (37.0 - CAST(s.reps AS FLOAT))),
                        2
                    ) as estimated_1rm
                FROM exercises e
                JOIN sets s ON e.id = s.exercise_id
                WHERE LOWER(e.exercise_type) = LOWER(?)
            )
            SELECT 
                COALESCE(MAX(weight), 0.0) as "all_time_max_weight!: f64",
                COALESCE(MAX(volume), 0.0) as "max_volume!: f64",
                COALESCE(MAX(estimated_1rm), 0.0) as "estimated_max_1rm!: f64",
                COALESCE(GROUP_CONCAT(CAST(reps AS TEXT) || ':' || CAST(weight AS TEXT)), '') as "rep_prs!: String"
            FROM exercise_stats
            "#,
            exercise_type
        )
        .fetch_one(&pool)
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
                    "total_sets": row.total_sets,
                    "max_estimated_1rm": row.max_estimated_1rm,
                    "set_schemes": if !row.set_schemes.is_empty() {
                        Some(row.set_schemes.split(',').collect::<Vec<_>>())
                    } else {
                        None
                    }
                })
            }).collect::<Vec<_>>(),
            "monthly_volume": monthly_volume.iter().map(|row| {
                json!({
                    "month": row.month,
                    "total_volume": row.total_volume,
                    "max_weight": row.max_weight,
                    "total_reps": row.total_reps,
                    "total_sets": row.total_sets
                })
            }).collect::<Vec<_>>(),
            "personal_records": {
                "all_time_max_weight": personal_records.all_time_max_weight,
                "max_volume": personal_records.max_volume,
                "estimated_max_1rm": personal_records.estimated_max_1rm,
                "rep_prs": if personal_records.rep_prs.is_empty() {
                    None
                } else {
                    Some(personal_records.rep_prs.split(',')
                        .filter_map(|pair| {
                            let parts: Vec<&str> = pair.split(':').collect();
                            if parts.len() == 2 {
                                Some(json!({
                                    "reps": parts[0].parse::<i64>().unwrap_or(0),
                                    "weight": parts[1].parse::<f64>().unwrap_or(0.0)
                                }))
                            } else {
                                None
                            }
                        })
                        .collect::<Vec<_>>())
                }
            }
        }))
    }
}
