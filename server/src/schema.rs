use crate::backup;
use chrono::{Datelike, Duration, NaiveDate, TimeZone, Utc};
use sqlx::Row;
use sqlx::{Pool, Sqlite};

pub const INITIAL_SCHEMA: &str = r#"
-- Enable foreign key support
PRAGMA foreign_keys = ON;

-- Create workouts table
CREATE TABLE IF NOT EXISTS workouts (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    date DATETIME NOT NULL,
    start_time DATETIME NOT NULL,
    end_time DATETIME NOT NULL,
    notes TEXT,
    feedback TEXT CHECK(feedback IN ('😊', '😐', '😞') OR feedback IS NULL)
);

-- Create exercises table
CREATE TABLE IF NOT EXISTS exercises (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    workout_id INTEGER NOT NULL,
    exercise_type TEXT NOT NULL,
    start_time DATETIME NOT NULL,
    end_time DATETIME NOT NULL,
    notes TEXT,
    FOREIGN KEY (workout_id) REFERENCES workouts(id) ON DELETE CASCADE
);

-- Create sets table
CREATE TABLE IF NOT EXISTS sets (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    exercise_id INTEGER NOT NULL,
    reps INTEGER NOT NULL,
    weight REAL NOT NULL,
    notes TEXT,
    FOREIGN KEY (exercise_id) REFERENCES exercises(id) ON DELETE CASCADE
);

-- Create indexes for better performance
CREATE INDEX IF NOT EXISTS idx_exercises_workout_id_composite ON exercises(workout_id, id);
CREATE INDEX IF NOT EXISTS idx_sets_exercise_id_composite ON sets(exercise_id, id);
"#;

pub const SCHEMA_UPDATES: &[(i64, &str)] = &[
    (
        2,
        r#"
        ALTER TABLE exercises ADD COLUMN per_side_weight INTEGER NOT NULL DEFAULT 0;
        ALTER TABLE exercises ADD COLUMN split_weight INTEGER NOT NULL DEFAULT 0;
        ALTER TABLE sets ADD COLUMN weight_left REAL;
        ALTER TABLE sets ADD COLUMN weight_right REAL;

        CREATE TABLE IF NOT EXISTS exercise_settings (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            exercise_id INTEGER NOT NULL,
            setting_key TEXT NOT NULL,
            setting_value TEXT NOT NULL,
            FOREIGN KEY (exercise_id) REFERENCES exercises(id) ON DELETE CASCADE
        );

        CREATE INDEX IF NOT EXISTS idx_exercise_settings_exercise_id_composite
            ON exercise_settings(exercise_id, id);
        "#,
    ),
    (
        3,
        r#"
        ALTER TABLE workouts ADD COLUMN timezone_offset_minutes INTEGER;
        "#,
    ),
    (
        4,
        r#"
        -- Backfill workout timezone offsets for legacy data (Europe/Amsterdam).
        -- Applied via Rust migration logic in `setup_schema` (kept as a placeholder for versioning).
        SELECT 1;
        "#,
    ),
];

pub const SCHEMA_VERSION_TABLE: &str = r#"
CREATE TABLE IF NOT EXISTS schema_version (
    version INTEGER PRIMARY KEY,
    applied_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
);
"#;

pub async fn setup_schema(pool: &sqlx::Pool<sqlx::Sqlite>) -> Result<(), sqlx::Error> {
    sqlx::query(SCHEMA_VERSION_TABLE).execute(pool).await?;

    let has_version_table = sqlx::query_scalar!(
        r#"SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='schema_version'"#
    )
    .fetch_one(pool)
    .await?
        > 0;

    let needs_schema_update = if !has_version_table {
        true
    } else {
        sqlx::query_scalar!(r#"SELECT COUNT(*) FROM schema_version WHERE version = 1"#)
            .fetch_one(pool)
            .await?
            == 0
    };

    if needs_schema_update {
        let has_workouts = sqlx::query_scalar!(
            r#"SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='workouts'"#
        )
        .fetch_one(pool)
        .await?
            > 0;

        if has_workouts {
            backup::create_backup(backup::BackupType::Auto)
                .await
                .map_err(|e| sqlx::Error::Protocol(format!("Failed to create backup: {}", e)))?;
        }

        sqlx::query(INITIAL_SCHEMA).execute(pool).await?;

        sqlx::query("INSERT OR IGNORE INTO schema_version (version) VALUES (1)")
            .execute(pool)
            .await?;
    }

    for (version, update_sql) in SCHEMA_UPDATES {
        let version_exists = sqlx::query_scalar!(
            r#"SELECT COUNT(*) FROM schema_version WHERE version = ?"#,
            *version
        )
        .fetch_one(pool)
        .await?
            > 0;

        if !version_exists {
            if *version == 3 {
                let column_exists: i64 = sqlx::query_scalar(
                    "SELECT COUNT(*) FROM pragma_table_info('workouts') WHERE name = 'timezone_offset_minutes'",
                )
                .fetch_one(pool)
                .await?;

                if column_exists > 0 {
                    sqlx::query("INSERT INTO schema_version (version) VALUES (?)")
                        .bind(*version)
                        .execute(pool)
                        .await?;
                    continue;
                }
            }

            if *version == 4 {
                let has_offset_column: i64 = sqlx::query_scalar(
                    "SELECT COUNT(*) FROM pragma_table_info('workouts') WHERE name = 'timezone_offset_minutes'",
                )
                .fetch_one(pool)
                .await?;
                if has_offset_column == 0 {
                    return Err(sqlx::Error::Protocol(
                        "schema v4 requires workouts.timezone_offset_minutes column".into(),
                    ));
                }
            }

            let has_workouts_table = sqlx::query_scalar!(
                r#"SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='workouts'"#
            )
            .fetch_one(pool)
            .await?
                > 0;

            if has_workouts_table {
                let workout_count = sqlx::query_scalar!(r#"SELECT COUNT(*) FROM workouts"#)
                    .fetch_one(pool)
                    .await?;
                if workout_count > 0 {
                    backup::create_backup(backup::BackupType::Auto)
                        .await
                        .map_err(|e| {
                            sqlx::Error::Protocol(format!("Failed to create backup: {}", e))
                        })?;
                }
            }

            if *version == 4 {
                backfill_amsterdam_timezone_offsets(pool).await?;
                sqlx::query("INSERT INTO schema_version (version) VALUES (?)")
                    .bind(*version)
                    .execute(pool)
                    .await?;
                continue;
            }

            sqlx::query(update_sql).execute(pool).await?;

            sqlx::query("INSERT INTO schema_version (version) VALUES (?)")
                .bind(*version)
                .execute(pool)
                .await?;
        }
    }

    Ok(())
}

fn eu_dst_window_utc(year: i32) -> (chrono::DateTime<Utc>, chrono::DateTime<Utc>) {
    fn last_sunday(year: i32, month: u32) -> NaiveDate {
        let first_next = if month == 12 {
            NaiveDate::from_ymd_opt(year + 1, 1, 1).expect("next year jan 1")
        } else {
            NaiveDate::from_ymd_opt(year, month + 1, 1).expect("next month day 1")
        };
        let last_day = first_next - Duration::days(1);
        let offset = last_day.weekday().num_days_from_sunday() as i64;
        last_day - Duration::days(offset)
    }

    // EU DST: last Sunday of March 01:00 UTC -> last Sunday of October 01:00 UTC.
    let start_day = last_sunday(year, 3);
    let end_day = last_sunday(year, 10);
    let start = Utc.from_utc_datetime(&start_day.and_hms_opt(1, 0, 0).expect("start time"));
    let end = Utc.from_utc_datetime(&end_day.and_hms_opt(1, 0, 0).expect("end time"));
    (start, end)
}

fn amsterdam_timezone_offset_minutes_utc(utc: chrono::DateTime<Utc>) -> i64 {
    let (start, end) = eu_dst_window_utc(utc.year());
    if utc >= start && utc < end {
        // Amsterdam summer time (CEST, UTC+2). JS getTimezoneOffset() is negative for UTC+.
        -120
    } else {
        // Amsterdam standard time (CET, UTC+1).
        -60
    }
}

async fn backfill_amsterdam_timezone_offsets(pool: &Pool<Sqlite>) -> Result<(), sqlx::Error> {
    let mut tx = pool.begin().await?;

    let rows =
        sqlx::query("SELECT id, start_time FROM workouts WHERE timezone_offset_minutes IS NULL")
            .fetch_all(&mut *tx)
            .await?;

    for row in rows {
        let id: i64 = row.try_get("id")?;
        let start_time: chrono::DateTime<Utc> = row.try_get("start_time")?;
        let offset = amsterdam_timezone_offset_minutes_utc(start_time);

        sqlx::query(
            "UPDATE workouts SET timezone_offset_minutes = ? WHERE id = ? AND timezone_offset_minutes IS NULL",
        )
        .bind(offset)
        .bind(id)
        .execute(&mut *tx)
        .await?;
    }

    tx.commit().await?;
    Ok(())
}
