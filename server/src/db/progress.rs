use super::Database;
use crate::{errors::AppError, models::*};
use chrono::{DateTime, Duration, Utc};
use log::{debug, error};
use serde_json::json;
use sqlx::Row;

const MAX_TIMEZONE_OFFSET_MINUTES: i64 = 14 * 60;
const RECENT_PR_LIMIT: usize = 20;
const RECENT_BEST_WINDOW_DAYS: i64 = 90;

#[derive(Debug, Clone)]
struct SetFact {
    exercise_type: String,
    exercise_start_time: DateTime<Utc>,
    set_id: i64,
    reps: i64,
    effective_weight: f64,
    duration_seconds: Option<i64>,
}

impl SetFact {
    fn set_volume(&self) -> f64 {
        self.reps as f64 * self.effective_weight
    }

    fn estimated_1rm(&self) -> Option<f64> {
        if !(1..=12).contains(&self.reps) || self.effective_weight <= 0.0 {
            return None;
        }
        Some((self.effective_weight * (36.0 / (37.0 - self.reps as f64)) * 100.0).round() / 100.0)
    }
}

#[derive(Debug, Clone)]
struct PrEvent {
    exercise_type: String,
    pr_type: &'static str,
    occurred_at: DateTime<Utc>,
    set_id: i64,
    new_value: f64,
    previous_value: f64,
    reps: i64,
    weight: f64,
    duration_seconds: Option<i64>,
}

#[derive(Debug, Clone)]
struct PeriodSummaryBase {
    workouts: i64,
    total_training_minutes: i64,
    exercises: i64,
    sets: i64,
    reps: i64,
    total_volume: f64,
    timed_sets: i64,
    total_timed_duration_seconds: i64,
}

fn validate_progress_offset(offset: i64) -> Result<(), AppError> {
    if !(-MAX_TIMEZONE_OFFSET_MINUTES..=MAX_TIMEZONE_OFFSET_MINUTES).contains(&offset) {
        return Err(AppError::BadRequest(
            "timezone_offset_minutes is out of range".to_string(),
        ));
    }
    Ok(())
}

fn count_prs_in_period(events: &[PrEvent], start: DateTime<Utc>, end: DateTime<Utc>) -> i64 {
    events
        .iter()
        .filter(|event| event.occurred_at >= start && event.occurred_at < end)
        .count() as i64
}

fn event_json(event: &PrEvent) -> serde_json::Value {
    json!({
        "exercise_type": event.exercise_type,
        "pr_type": event.pr_type,
        "new_value": event.new_value,
        "previous_value": event.previous_value,
        "date": event.occurred_at,
        "set_id": event.set_id,
        "set_details": {
            "reps": event.reps,
            "weight": event.weight,
            "duration_seconds": event.duration_seconds
        }
    })
}

fn detect_pr_events(facts: &[SetFact]) -> Vec<PrEvent> {
    use std::collections::HashMap;

    let mut max_weight_by_exercise: HashMap<String, f64> = HashMap::new();
    let mut one_rm_by_exercise: HashMap<String, f64> = HashMap::new();
    let mut volume_by_exercise: HashMap<String, f64> = HashMap::new();
    let mut duration_by_exercise: HashMap<String, f64> = HashMap::new();
    let mut rep_by_exercise_and_reps: HashMap<(String, i64), f64> = HashMap::new();
    let mut events = Vec::new();

    for fact in facts {
        let mut set_events = Vec::new();

        maybe_record_pr(
            &mut max_weight_by_exercise,
            fact.exercise_type.clone(),
            fact.effective_weight,
            fact,
            "max_weight",
            &mut set_events,
        );

        if let Some(estimated_1rm) = fact.estimated_1rm() {
            maybe_record_pr(
                &mut one_rm_by_exercise,
                fact.exercise_type.clone(),
                estimated_1rm,
                fact,
                "estimated_1rm",
                &mut set_events,
            );
        }

        if fact.reps > 0 && fact.effective_weight > 0.0 {
            let key = (fact.exercise_type.clone(), fact.reps);
            let previous = rep_by_exercise_and_reps.insert(
                key.clone(),
                rep_by_exercise_and_reps
                    .get(&key)
                    .copied()
                    .map_or(fact.effective_weight, |current| {
                        current.max(fact.effective_weight)
                    }),
            );
            if let Some(previous_value) = previous {
                if fact.effective_weight > previous_value {
                    set_events.push(pr_event(
                        fact,
                        "rep_pr",
                        fact.effective_weight,
                        previous_value,
                    ));
                }
            }
        }

        maybe_record_pr(
            &mut volume_by_exercise,
            fact.exercise_type.clone(),
            fact.set_volume(),
            fact,
            "single_set_volume",
            &mut set_events,
        );

        if let Some(duration) = fact.duration_seconds.filter(|d| *d > 0) {
            maybe_record_pr(
                &mut duration_by_exercise,
                fact.exercise_type.clone(),
                duration as f64,
                fact,
                "timed_duration",
                &mut set_events,
            );
        }

        if let Some(best_event) = set_events.into_iter().min_by_key(pr_priority) {
            events.push(best_event);
        }
    }

    events.sort_by(|a, b| {
        a.occurred_at
            .cmp(&b.occurred_at)
            .then_with(|| a.set_id.cmp(&b.set_id))
    });
    events
}

fn detect_recent_best_events(facts: &[SetFact]) -> Vec<PrEvent> {
    use std::collections::HashMap;

    let mut histories: HashMap<String, Vec<(DateTime<Utc>, f64)>> = HashMap::new();
    let mut events = Vec::new();

    for fact in facts {
        let mut set_events = Vec::new();

        maybe_record_recent_best(
            &mut histories,
            format!("{}::max_weight", fact.exercise_type),
            fact.effective_weight,
            fact,
            "max_weight",
            &mut set_events,
        );

        if let Some(estimated_1rm) = fact.estimated_1rm() {
            maybe_record_recent_best(
                &mut histories,
                format!("{}::estimated_1rm", fact.exercise_type),
                estimated_1rm,
                fact,
                "estimated_1rm",
                &mut set_events,
            );
        }

        if fact.reps > 0 && fact.effective_weight > 0.0 {
            maybe_record_recent_best(
                &mut histories,
                format!("{}::rep_pr::{}", fact.exercise_type, fact.reps),
                fact.effective_weight,
                fact,
                "rep_pr",
                &mut set_events,
            );
        }

        maybe_record_recent_best(
            &mut histories,
            format!("{}::single_set_volume", fact.exercise_type),
            fact.set_volume(),
            fact,
            "single_set_volume",
            &mut set_events,
        );

        if let Some(duration) = fact.duration_seconds.filter(|d| *d > 0) {
            maybe_record_recent_best(
                &mut histories,
                format!("{}::timed_duration", fact.exercise_type),
                duration as f64,
                fact,
                "timed_duration",
                &mut set_events,
            );
        }

        if let Some(best_event) = set_events.into_iter().min_by_key(pr_priority) {
            events.push(best_event);
        }
    }

    events.sort_by(|a, b| {
        a.occurred_at
            .cmp(&b.occurred_at)
            .then_with(|| a.set_id.cmp(&b.set_id))
    });
    events
}

fn maybe_record_recent_best(
    histories: &mut std::collections::HashMap<String, Vec<(DateTime<Utc>, f64)>>,
    key: String,
    value: f64,
    fact: &SetFact,
    pr_type: &'static str,
    events: &mut Vec<PrEvent>,
) {
    if !value.is_finite() || value <= 0.0 {
        return;
    }

    let window_start = fact.exercise_start_time - Duration::days(RECENT_BEST_WINDOW_DAYS);
    let history = histories.entry(key).or_default();
    history.retain(|(occurred_at, _)| *occurred_at >= window_start);

    let previous_best = history
        .iter()
        .map(|(_, previous)| *previous)
        .reduce(f64::max);

    if let Some(previous_value) = previous_best {
        if value > previous_value {
            events.push(pr_event(fact, pr_type, value, previous_value));
        }
    }

    history.push((fact.exercise_start_time, value));
}

fn maybe_record_pr(
    bests: &mut std::collections::HashMap<String, f64>,
    key: String,
    value: f64,
    fact: &SetFact,
    pr_type: &'static str,
    events: &mut Vec<PrEvent>,
) {
    if !value.is_finite() || value <= 0.0 {
        return;
    }

    match bests.get_mut(&key) {
        Some(best) if value > *best => {
            let previous = *best;
            *best = value;
            events.push(pr_event(fact, pr_type, value, previous));
        }
        Some(_) => {}
        None => {
            bests.insert(key, value);
        }
    }
}

fn pr_event(fact: &SetFact, pr_type: &'static str, new_value: f64, previous_value: f64) -> PrEvent {
    PrEvent {
        exercise_type: fact.exercise_type.clone(),
        pr_type,
        occurred_at: fact.exercise_start_time,
        set_id: fact.set_id,
        new_value,
        previous_value,
        reps: fact.reps,
        weight: fact.effective_weight,
        duration_seconds: fact.duration_seconds,
    }
}

fn pr_priority(event: &PrEvent) -> i32 {
    if event.duration_seconds.filter(|d| *d > 0).is_some() && event.reps == 0 {
        return match event.pr_type {
            "timed_duration" => 0,
            _ => 10,
        };
    }

    match event.pr_type {
        "estimated_1rm" => 0,
        "max_weight" => 1,
        "rep_pr" => 2,
        "single_set_volume" => 3,
        "timed_duration" => 4,
        _ => 10,
    }
}

impl Database {
    pub async fn get_exercise_progress(
        &self,
        user_id: i64,
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
            WHERE user_id = ?
              AND LOWER(exercise_type) = LOWER(?)
            ORDER BY start_time ASC
            "#,
            user_id,
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
            result.push((exercise, sets));
        }

        debug!(target: "database", "Found {} exercises for progress data", result.len());
        Ok(result)
    }

    pub async fn get_workout_stats(&self, user_id: i64) -> Result<serde_json::Value, AppError> {
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
                WHERE user_id = ?
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
            ,
            user_id
        )
        .fetch_one(&pool)
        .await
        .map_err(|e| {
            error!(target: "database", "Failed to fetch workout stats: {}", e);
            AppError::DatabaseError(e)
        })?;

        let popular_hours_raw = stats.popular_hours.unwrap_or_default();
        let popular_hours = if popular_hours_raw.is_empty() {
            Vec::new()
        } else {
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
                .collect::<Vec<_>>()
        };

        let duration_distribution_raw = stats.duration_distribution.unwrap_or_default();
        let duration_distribution = if duration_distribution_raw.is_empty() {
            Vec::new()
        } else {
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
                .collect::<Vec<_>>()
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
                WHERE user_id = ?
                  AND date(start_time) >= date('now', 'start of month', '-11 months')
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
        .bind(user_id)
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
                          AND e.user_id = workouts.user_id
                    ) as exercise_count
                FROM workouts
                WHERE user_id = ?
                  AND end_time > start_time
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
            ,
            user_id
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
              AND user_id = ?
              AND date(start_time) >= date('now', 'start of month', '-11 months')
            ORDER BY start_time ASC
            "#,
            user_id
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

    pub async fn get_progress_overview(
        &self,
        user_id: i64,
        timezone_offset_minutes: i64,
    ) -> Result<serde_json::Value, AppError> {
        validate_progress_offset(timezone_offset_minutes)?;

        let now = Utc::now();
        let last_7_end = now;
        let last_7_start = now - Duration::days(7);
        let previous_7_start = last_7_start - Duration::days(7);
        let last_30_end = now;
        let last_30_start = now - Duration::days(30);
        let previous_30_start = last_30_start - Duration::days(30);

        let facts = self.get_progress_set_facts(user_id).await?;
        let pr_events = detect_pr_events(&facts);
        let recent_best_events = detect_recent_best_events(&facts);

        let last_7_pr_count = count_prs_in_period(&pr_events, last_7_start, last_7_end);
        let previous_7_pr_count = count_prs_in_period(&pr_events, previous_7_start, last_7_start);
        let last_30_pr_count = count_prs_in_period(&pr_events, last_30_start, last_30_end);
        let previous_30_pr_count =
            count_prs_in_period(&pr_events, previous_30_start, last_30_start);

        let last_7_recent_best_count =
            count_prs_in_period(&recent_best_events, last_7_start, last_7_end);
        let previous_7_recent_best_count =
            count_prs_in_period(&recent_best_events, previous_7_start, last_7_start);
        let last_30_recent_best_count =
            count_prs_in_period(&recent_best_events, last_30_start, last_30_end);
        let previous_30_recent_best_count =
            count_prs_in_period(&recent_best_events, previous_30_start, last_30_start);

        let last_7_days = self
            .get_period_summary(
                user_id,
                "Last 7 days",
                last_7_start,
                last_7_end,
                previous_7_start,
                last_7_start,
                last_7_pr_count,
                previous_7_pr_count,
                last_7_recent_best_count,
                previous_7_recent_best_count,
            )
            .await?;
        let last_30_days = self
            .get_period_summary(
                user_id,
                "Last 30 days",
                last_30_start,
                last_30_end,
                previous_30_start,
                last_30_start,
                last_30_pr_count,
                previous_30_pr_count,
                last_30_recent_best_count,
                previous_30_recent_best_count,
            )
            .await?;

        let recent_prs = pr_events
            .iter()
            .rev()
            .take(RECENT_PR_LIMIT)
            .map(event_json)
            .collect::<Vec<_>>();

        let recent_bests = recent_best_events
            .iter()
            .rev()
            .take(RECENT_PR_LIMIT)
            .map(event_json)
            .collect::<Vec<_>>();

        Ok(json!({
            "last_7_days": last_7_days,
            "last_30_days": last_30_days,
            "recent_prs": recent_prs,
            "recent_bests": recent_bests
        }))
    }

    async fn get_period_summary(
        &self,
        user_id: i64,
        label: &str,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
        comparison_start: DateTime<Utc>,
        comparison_end: DateTime<Utc>,
        pr_count: i64,
        comparison_pr_count: i64,
        recent_best_count: i64,
        comparison_recent_best_count: i64,
    ) -> Result<serde_json::Value, AppError> {
        let current = self.get_period_summary_base(user_id, start, end).await?;
        let previous = self
            .get_period_summary_base(user_id, comparison_start, comparison_end)
            .await?;

        Ok(json!({
            "label": label,
            "start_date": start,
            "end_date": end,
            "workouts": current.workouts,
            "total_training_minutes": current.total_training_minutes,
            "exercises": current.exercises,
            "sets": current.sets,
            "reps": current.reps,
            "total_volume": current.total_volume,
            "timed_sets": current.timed_sets,
            "total_timed_duration_seconds": current.total_timed_duration_seconds,
            "pr_count": pr_count,
            "recent_best_count": recent_best_count,
            "comparison": {
                "workouts_delta": current.workouts - previous.workouts,
                "total_training_minutes_delta": current.total_training_minutes - previous.total_training_minutes,
                "exercises_delta": current.exercises - previous.exercises,
                "sets_delta": current.sets - previous.sets,
                "reps_delta": current.reps - previous.reps,
                "total_volume_delta": current.total_volume - previous.total_volume,
                "timed_sets_delta": current.timed_sets - previous.timed_sets,
                "total_timed_duration_seconds_delta": current.total_timed_duration_seconds - previous.total_timed_duration_seconds,
                "pr_count_delta": pr_count - comparison_pr_count,
                "recent_best_count_delta": recent_best_count - comparison_recent_best_count
            }
        }))
    }

    async fn get_period_summary_base(
        &self,
        user_id: i64,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> Result<PeriodSummaryBase, AppError> {
        let pool = self.pool().await;
        let row = sqlx::query(
            r#"
            WITH period_workouts AS (
                SELECT id, start_time, end_time
                FROM workouts
                WHERE user_id = ?
                  AND start_time >= ?
                  AND start_time < ?
            ),
            period_exercises AS (
                SELECT e.id, e.per_side_weight, e.split_weight
                FROM exercises e
                JOIN period_workouts w ON w.id = e.workout_id
                WHERE e.user_id = ?
            ),
            period_sets AS (
                SELECT
                    s.reps,
                    s.duration_seconds,
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
                    ) as volume
                FROM sets s
                JOIN period_exercises e ON e.id = s.exercise_id
                WHERE s.user_id = ?
            )
            SELECT
                (SELECT COUNT(*) FROM period_workouts) as workouts,
                COALESCE((
                    SELECT CAST(ROUND(SUM((julianday(end_time) - julianday(start_time)) * 24 * 60)) AS INTEGER)
                    FROM period_workouts
                    WHERE end_time > start_time
                ), 0) as total_training_minutes,
                (SELECT COUNT(*) FROM period_exercises) as exercises,
                (SELECT COUNT(*) FROM period_sets) as sets,
                COALESCE((SELECT SUM(reps) FROM period_sets), 0) as reps,
                COALESCE((SELECT SUM(volume) FROM period_sets), 0.0) as total_volume,
                COALESCE((SELECT COUNT(*) FROM period_sets WHERE duration_seconds IS NOT NULL AND duration_seconds > 0), 0) as timed_sets,
                COALESCE((SELECT SUM(duration_seconds) FROM period_sets WHERE duration_seconds IS NOT NULL AND duration_seconds > 0), 0) as total_timed_duration_seconds
            "#,
        )
        .bind(user_id)
        .bind(start)
        .bind(end)
        .bind(user_id)
        .bind(user_id)
        .fetch_one(&pool)
        .await
        .map_err(|e| {
            error!(target: "database", "Failed to fetch period summary: {}", e);
            AppError::DatabaseError(e)
        })?;

        Ok(PeriodSummaryBase {
            workouts: row.try_get("workouts").unwrap_or(0),
            total_training_minutes: row.try_get("total_training_minutes").unwrap_or(0),
            exercises: row.try_get("exercises").unwrap_or(0),
            sets: row.try_get("sets").unwrap_or(0),
            reps: row.try_get("reps").unwrap_or(0),
            total_volume: row.try_get("total_volume").unwrap_or(0.0),
            timed_sets: row.try_get("timed_sets").unwrap_or(0),
            total_timed_duration_seconds: row.try_get("total_timed_duration_seconds").unwrap_or(0),
        })
    }

    async fn get_progress_set_facts(&self, user_id: i64) -> Result<Vec<SetFact>, AppError> {
        let pool = self.pool().await;
        let rows = sqlx::query(
            r#"
            SELECT
                e.exercise_type,
                e.start_time as exercise_start_time,
                s.id as set_id,
                s.reps,
                CASE
                    WHEN e.per_side_weight = 1 THEN
                        CASE
                            WHEN e.split_weight = 1 AND s.weight_left IS NOT NULL AND s.weight_right IS NOT NULL
                                THEN (s.weight_left + s.weight_right)
                            ELSE (s.weight * 2)
                        END
                    ELSE s.weight
                END as effective_weight,
                s.duration_seconds
            FROM sets s
            JOIN exercises e ON e.id = s.exercise_id
            JOIN workouts w ON w.id = e.workout_id
            WHERE s.user_id = ?
              AND e.user_id = ?
              AND w.user_id = ?
            ORDER BY e.start_time ASC, s.id ASC
            "#,
        )
        .bind(user_id)
        .bind(user_id)
        .bind(user_id)
        .fetch_all(&pool)
        .await
        .map_err(|e| {
            error!(target: "database", "Failed to fetch progress set facts: {}", e);
            AppError::DatabaseError(e)
        })?;

        rows.into_iter()
            .map(|row| {
                Ok(SetFact {
                    exercise_type: row.try_get("exercise_type")?,
                    exercise_start_time: row.try_get("exercise_start_time")?,
                    set_id: row.try_get("set_id")?,
                    reps: row.try_get("reps")?,
                    effective_weight: row.try_get("effective_weight")?,
                    duration_seconds: row.try_get("duration_seconds")?,
                })
            })
            .collect::<Result<Vec<_>, sqlx::Error>>()
            .map_err(|e| {
                error!(target: "database", "Failed to map progress set facts: {}", e);
                AppError::DatabaseError(e)
            })
    }

    pub async fn get_volume_stats(
        &self,
        user_id: i64,
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
                WHERE e.user_id = ?
                  AND LOWER(e.exercise_type) = LOWER(?)
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
            user_id,
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
            WHERE e.user_id = ?
              AND LOWER(e.exercise_type) = LOWER(?)
            GROUP BY strftime('%Y-%m', e.start_time)
            ORDER BY month ASC
            "#,
            user_id,
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
                WHERE e.user_id = ?
                  AND LOWER(e.exercise_type) = LOWER(?)
            )
            SELECT 
                COALESCE(MAX(weight), 0.0) as "all_time_max_weight!: f64",
                COALESCE(MAX(volume), 0.0) as "max_volume!: f64",
                COALESCE(MAX(estimated_1rm), 0.0) as "estimated_max_1rm!: f64",
                COALESCE(GROUP_CONCAT(CAST(reps AS TEXT) || ':' || CAST(weight AS TEXT)), '') as "rep_prs!: String"
            FROM exercise_stats
            "#,
            user_id,
            exercise_type
        )
        .fetch_one(&pool)
        .await
        .map_err(|e| {
            error!(target: "database", "Failed to fetch PRs: {}", e);
            AppError::DatabaseError(e)
        })?;

        let timed_records = sqlx::query(
            r#"
            WITH timed_sets AS (
                SELECT
                    e.id as exercise_id,
                    s.duration_seconds
                FROM exercises e
                JOIN sets s ON e.id = s.exercise_id
                WHERE e.user_id = ?
                  AND s.user_id = ?
                  AND LOWER(e.exercise_type) = LOWER(?)
                  AND s.duration_seconds IS NOT NULL
                  AND s.duration_seconds > 0
            ),
            session_totals AS (
                SELECT exercise_id, SUM(duration_seconds) as session_duration_seconds
                FROM timed_sets
                GROUP BY exercise_id
            )
            SELECT
                COALESCE(MAX(duration_seconds), 0) as longest_set_seconds,
                COALESCE((SELECT MAX(session_duration_seconds) FROM session_totals), 0) as best_session_duration_seconds,
                COALESCE(SUM(duration_seconds), 0) as lifetime_duration_seconds,
                COALESCE(CAST(ROUND(AVG(duration_seconds)) AS INTEGER), 0) as average_set_duration_seconds,
                COUNT(*) as timed_set_count
            FROM timed_sets
            "#,
        )
        .bind(user_id)
        .bind(user_id)
        .bind(exercise_type)
        .fetch_one(&pool)
        .await
        .map_err(|e| {
            error!(target: "database", "Failed to fetch timed records: {}", e);
            AppError::DatabaseError(e)
        })?;

        let timed_set_count = timed_records
            .try_get::<i64, _>("timed_set_count")
            .unwrap_or(0);
        let timed_records_json = if timed_set_count > 0 {
            Some(json!({
                "longest_set_seconds": timed_records.try_get::<i64, _>("longest_set_seconds").unwrap_or(0),
                "best_session_duration_seconds": timed_records.try_get::<i64, _>("best_session_duration_seconds").unwrap_or(0),
                "lifetime_duration_seconds": timed_records.try_get::<i64, _>("lifetime_duration_seconds").unwrap_or(0),
                "average_set_duration_seconds": timed_records.try_get::<i64, _>("average_set_duration_seconds").unwrap_or(0)
            }))
        } else {
            None
        };

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
            },
            "timed_records": timed_records_json
        }))
    }
}
