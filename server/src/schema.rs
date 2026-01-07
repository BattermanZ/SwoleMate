use crate::backup;

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

            sqlx::query(update_sql).execute(pool).await?;

            sqlx::query("INSERT INTO schema_version (version) VALUES (?)")
                .bind(*version)
                .execute(pool)
                .await?;
        }
    }

    Ok(())
}

