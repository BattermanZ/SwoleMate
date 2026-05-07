use super::Database;
use crate::errors::AppError;
use chrono::{DateTime, Datelike, Duration, NaiveDate, Utc};
use log::{debug, error};
use sqlx::Row;
use std::collections::BTreeMap;

#[derive(Debug, Clone, Copy)]
pub struct CalendarWorkoutFrequency {
    pub average_per_week: f64,
    pub trend: f64,
}

fn round_one(value: f64) -> f64 {
    (value * 10.0).round() / 10.0
}

fn week_start(date: NaiveDate) -> NaiveDate {
    date - Duration::days(date.weekday().num_days_from_monday() as i64)
}

impl Database {
    pub async fn get_calendar_workout_frequency(
        &self,
        user_id: i64,
    ) -> Result<CalendarWorkoutFrequency, AppError> {
        debug!(target: "database", "Calculating calendar workout frequency");

        let pool = self.pool().await;
        let rows = sqlx::query(
            r#"
            SELECT start_time
            FROM workouts
            WHERE user_id = ?
              AND end_time > start_time
            ORDER BY start_time ASC
            "#,
        )
        .bind(user_id)
        .fetch_all(&pool)
        .await
        .map_err(|e| {
            error!(target: "database", "Failed to fetch workout frequency rows: {}", e);
            AppError::DatabaseError(e)
        })?;

        if rows.is_empty() {
            return Ok(CalendarWorkoutFrequency {
                average_per_week: 0.0,
                trend: 0.0,
            });
        }

        let mut counts_by_week = BTreeMap::<NaiveDate, i64>::new();

        for row in rows {
            let start_time: DateTime<Utc> = row.try_get("start_time").map_err(|e| {
                error!(target: "database", "Failed to read workout start_time: {}", e);
                AppError::DatabaseError(e)
            })?;
            *counts_by_week.entry(week_start(start_time.date_naive())).or_insert(0) += 1;
        }

        let first_week = *counts_by_week
            .keys()
            .next()
            .ok_or_else(|| AppError::InternalError("missing first workout week".to_string()))?;
        let current_week = week_start(Utc::now().date_naive());
        let week_count = ((current_week - first_week).num_days() / 7 + 1).max(1) as f64;
        let total_workouts = counts_by_week.values().sum::<i64>() as f64;
        let average_per_week = round_one(total_workouts / week_count);

        let recent_weeks = (1..=4)
            .map(|offset| current_week - Duration::days(offset * 7))
            .filter(|week| *week >= first_week)
            .collect::<Vec<_>>();

        let recent_average = if recent_weeks.is_empty() {
            0.0
        } else {
            let recent_total = recent_weeks
                .iter()
                .map(|week| *counts_by_week.get(week).unwrap_or(&0))
                .sum::<i64>() as f64;
            round_one(recent_total / recent_weeks.len() as f64)
        };

        Ok(CalendarWorkoutFrequency {
            average_per_week,
            trend: round_one(recent_average - average_per_week),
        })
    }
}
