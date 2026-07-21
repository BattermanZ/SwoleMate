use crate::auth::password;
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
    duration_seconds INTEGER,
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
    (
        5,
        r#"
        -- Add multi-user auth tables and scope workout data by user.
        -- Applied via Rust migration logic in `setup_schema` (kept as a placeholder for versioning).
        SELECT 1;
        "#,
    ),
    (
        6,
        r#"
        -- Ensure user_id foreign keys use ON DELETE CASCADE.
        -- Applied via Rust migration logic in `setup_schema` (kept as a placeholder for versioning).
        SELECT 1;
        "#,
    ),
    (
        7,
        r#"
        -- Add workout activity tracking + auto-close marker columns.
        -- Applied via Rust migration logic in `setup_schema` (kept as a placeholder for versioning).
        SELECT 1;
        "#,
    ),
    (
        8,
        r#"
        -- Require-password-change flag on users.
        -- Applied via Rust migration logic in `setup_schema` (kept as a placeholder for versioning).
        SELECT 1;
        "#,
    ),
    (
        9,
        r#"
        -- Add OAuth and MCP audit tables.
        -- Applied via Rust migration logic in `setup_schema` (kept as a placeholder for versioning).
        SELECT 1;
        "#,
    ),
    (
        10,
        r#"
        -- Enforce OAuth client foreign keys on dependent tables.
        -- Applied via Rust migration logic in `setup_schema` (kept as a placeholder for versioning).
        SELECT 1;
        "#,
    ),
    (
        11,
        r#"
        -- Add personal MCP token storage.
        -- Applied via Rust migration logic in `setup_schema` (kept as a placeholder for versioning).
        SELECT 1;
        "#,
    ),
    (
        12,
        r#"
        -- Add workout templates tables.
        -- Applied via Rust migration logic in `setup_schema` (kept as a placeholder for versioning).
        SELECT 1;
        "#,
    ),
    (
        13,
        r#"
        ALTER TABLE sets ADD COLUMN duration_seconds INTEGER;
        "#,
    ),
    (
        14,
        r#"
        -- Idempotency keys for offline-sync replay of the two non-idempotent POST
        -- creates (workout / exercise). A lost HTTP response used to duplicate the
        -- workout and its sets on reconnect replay; the client now sends a stable
        -- Idempotency-Key per offline entity and the server dedups on it.
        CREATE TABLE IF NOT EXISTS idempotency_keys (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            user_id INTEGER NOT NULL,
            request_kind TEXT NOT NULL,
            idempotency_key TEXT NOT NULL,
            resource_id INTEGER NOT NULL,
            created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
            FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE,
            UNIQUE(user_id, request_kind, idempotency_key)
        );
        "#,
    ),
    (
        15,
        r#"
        -- Token-family ids so rotated OAuth refresh tokens share a lineage. On
        -- replay of an already-rotated refresh token (reuse detection, RFC 6819
        -- 5.2.2.3) the whole family is revoked, killing a thief's descendant tokens.
        ALTER TABLE oauth_access_tokens ADD COLUMN family_id TEXT;
        ALTER TABLE oauth_refresh_tokens ADD COLUMN family_id TEXT;
        CREATE INDEX IF NOT EXISTS idx_oauth_access_tokens_family
            ON oauth_access_tokens(family_id);
        CREATE INDEX IF NOT EXISTS idx_oauth_refresh_tokens_family
            ON oauth_refresh_tokens(family_id);
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

            if *version == 5 {
                migrate_multi_user_schema(pool).await?;
                sqlx::query("INSERT INTO schema_version (version) VALUES (?)")
                    .bind(*version)
                    .execute(pool)
                    .await?;
                continue;
            }

            if *version == 6 {
                migrate_user_fk_cascades(pool).await?;
                sqlx::query("INSERT INTO schema_version (version) VALUES (?)")
                    .bind(*version)
                    .execute(pool)
                    .await?;
                continue;
            }

            if *version == 7 {
                migrate_workout_autoclose(pool).await?;
                sqlx::query("INSERT INTO schema_version (version) VALUES (?)")
                    .bind(*version)
                    .execute(pool)
                    .await?;
                continue;
            }

            if *version == 8 {
                migrate_user_must_change_password(pool).await?;
                sqlx::query("INSERT INTO schema_version (version) VALUES (?)")
                    .bind(*version)
                    .execute(pool)
                    .await?;
                continue;
            }

            if *version == 9 {
                migrate_oauth_and_mcp_foundation(pool).await?;
                sqlx::query("INSERT INTO schema_version (version) VALUES (?)")
                    .bind(*version)
                    .execute(pool)
                    .await?;
                continue;
            }

            if *version == 10 {
                migrate_oauth_client_foreign_keys(pool).await?;
                sqlx::query("INSERT INTO schema_version (version) VALUES (?)")
                    .bind(*version)
                    .execute(pool)
                    .await?;
                continue;
            }

            if *version == 11 {
                migrate_mcp_tokens(pool).await?;
                sqlx::query("INSERT INTO schema_version (version) VALUES (?)")
                    .bind(*version)
                    .execute(pool)
                    .await?;
                continue;
            }

            if *version == 12 {
                migrate_workout_templates(pool).await?;
                sqlx::query("INSERT INTO schema_version (version) VALUES (?)")
                    .bind(*version)
                    .execute(pool)
                    .await?;
                continue;
            }

            if *version == 13 {
                let column_exists: i64 = sqlx::query_scalar(
                    "SELECT COUNT(*) FROM pragma_table_info('sets') WHERE name = 'duration_seconds'",
                )
                .fetch_one(pool)
                .await?;

                if column_exists == 0 {
                    sqlx::query(update_sql).execute(pool).await?;
                }

                sqlx::query("INSERT INTO schema_version (version) VALUES (?)")
                    .bind(*version)
                    .execute(pool)
                    .await?;
                continue;
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

async fn migrate_workout_autoclose(pool: &Pool<Sqlite>) -> Result<(), sqlx::Error> {
    let has_workouts_table: i64 = sqlx::query_scalar(
        r#"SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='workouts'"#,
    )
    .fetch_one(pool)
    .await?;
    if has_workouts_table == 0 {
        return Ok(());
    }

    let workout_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM workouts")
        .fetch_one(pool)
        .await?;
    if workout_count > 0 {
        backup::create_backup(backup::BackupType::Auto)
            .await
            .map_err(|e| sqlx::Error::Protocol(format!("Failed to create backup: {}", e)))?;
    }

    let has_last_activity: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM pragma_table_info('workouts') WHERE name = 'last_activity_time'",
    )
    .fetch_one(pool)
    .await?;
    if has_last_activity == 0 {
        sqlx::query("ALTER TABLE workouts ADD COLUMN last_activity_time DATETIME")
            .execute(pool)
            .await?;
    }

    let has_auto_closed: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM pragma_table_info('workouts') WHERE name = 'auto_closed_at'",
    )
    .fetch_one(pool)
    .await?;
    if has_auto_closed == 0 {
        sqlx::query("ALTER TABLE workouts ADD COLUMN auto_closed_at DATETIME")
            .execute(pool)
            .await?;
    }

    sqlx::query(
        "UPDATE workouts SET last_activity_time = start_time WHERE last_activity_time IS NULL",
    )
    .execute(pool)
    .await?;

    Ok(())
}

async fn migrate_user_must_change_password(pool: &Pool<Sqlite>) -> Result<(), sqlx::Error> {
    let has_users_table: i64 = sqlx::query_scalar(
        r#"SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='users'"#,
    )
    .fetch_one(pool)
    .await?;
    if has_users_table == 0 {
        return Ok(());
    }

    let has_column: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM pragma_table_info('users') WHERE name = 'must_change_password'",
    )
    .fetch_one(pool)
    .await?;
    if has_column == 0 {
        sqlx::query("ALTER TABLE users ADD COLUMN must_change_password INTEGER NOT NULL DEFAULT 0")
            .execute(pool)
            .await?;
    }

    Ok(())
}

async fn migrate_oauth_and_mcp_foundation(pool: &Pool<Sqlite>) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS oauth_clients (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            client_id TEXT NOT NULL UNIQUE,
            client_secret_hash TEXT,
            client_name TEXT NOT NULL,
            redirect_uris_json TEXT NOT NULL,
            scopes_json TEXT NOT NULL,
            created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
            disabled_at DATETIME
        );

        CREATE INDEX IF NOT EXISTS idx_oauth_clients_client_id
            ON oauth_clients(client_id);

        CREATE TABLE IF NOT EXISTS oauth_authorization_codes (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            code_hash TEXT NOT NULL UNIQUE,
            client_id TEXT NOT NULL,
            user_id INTEGER NOT NULL,
            redirect_uri TEXT NOT NULL,
            scopes_json TEXT NOT NULL,
            pkce_code_challenge TEXT,
            pkce_method TEXT,
            expires_at DATETIME NOT NULL,
            used_at DATETIME,
            created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
            FOREIGN KEY (client_id) REFERENCES oauth_clients(client_id) ON DELETE CASCADE,
            FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
        );

        CREATE INDEX IF NOT EXISTS idx_oauth_authorization_codes_code_hash
            ON oauth_authorization_codes(code_hash);
        CREATE INDEX IF NOT EXISTS idx_oauth_authorization_codes_client_id
            ON oauth_authorization_codes(client_id);
        CREATE INDEX IF NOT EXISTS idx_oauth_authorization_codes_user_id
            ON oauth_authorization_codes(user_id);

        CREATE TABLE IF NOT EXISTS oauth_access_tokens (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            token_hash TEXT NOT NULL UNIQUE,
            client_id TEXT NOT NULL,
            user_id INTEGER NOT NULL,
            scopes_json TEXT NOT NULL,
            expires_at DATETIME NOT NULL,
            revoked_at DATETIME,
            created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
            FOREIGN KEY (client_id) REFERENCES oauth_clients(client_id) ON DELETE CASCADE,
            FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
        );

        CREATE INDEX IF NOT EXISTS idx_oauth_access_tokens_token_hash
            ON oauth_access_tokens(token_hash);
        CREATE INDEX IF NOT EXISTS idx_oauth_access_tokens_client_id
            ON oauth_access_tokens(client_id);
        CREATE INDEX IF NOT EXISTS idx_oauth_access_tokens_user_id
            ON oauth_access_tokens(user_id);

        CREATE TABLE IF NOT EXISTS oauth_refresh_tokens (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            token_hash TEXT NOT NULL UNIQUE,
            client_id TEXT NOT NULL,
            user_id INTEGER NOT NULL,
            scopes_json TEXT NOT NULL,
            expires_at DATETIME NOT NULL,
            revoked_at DATETIME,
            created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
            FOREIGN KEY (client_id) REFERENCES oauth_clients(client_id) ON DELETE CASCADE,
            FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
        );

        CREATE INDEX IF NOT EXISTS idx_oauth_refresh_tokens_token_hash
            ON oauth_refresh_tokens(token_hash);
        CREATE INDEX IF NOT EXISTS idx_oauth_refresh_tokens_client_id
            ON oauth_refresh_tokens(client_id);
        CREATE INDEX IF NOT EXISTS idx_oauth_refresh_tokens_user_id
            ON oauth_refresh_tokens(user_id);

        CREATE TABLE IF NOT EXISTS oauth_consents (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            user_id INTEGER NOT NULL,
            client_id TEXT NOT NULL,
            scopes_json TEXT NOT NULL,
            granted_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
            revoked_at DATETIME,
            FOREIGN KEY (client_id) REFERENCES oauth_clients(client_id) ON DELETE CASCADE,
            FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
        );

        CREATE INDEX IF NOT EXISTS idx_oauth_consents_user_client
            ON oauth_consents(user_id, client_id);

        CREATE TABLE IF NOT EXISTS mcp_audit_log (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            timestamp DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
            user_id INTEGER,
            client_id TEXT,
            tool_name TEXT NOT NULL,
            success INTEGER NOT NULL,
            error_code TEXT,
            input_summary_json TEXT,
            ip_address TEXT,
            user_agent TEXT,
            FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE SET NULL
        );

        CREATE INDEX IF NOT EXISTS idx_mcp_audit_log_timestamp
            ON mcp_audit_log(timestamp);
        CREATE INDEX IF NOT EXISTS idx_mcp_audit_log_user_id
            ON mcp_audit_log(user_id);
        CREATE INDEX IF NOT EXISTS idx_mcp_audit_log_client_id
            ON mcp_audit_log(client_id);
        "#,
    )
    .execute(pool)
    .await?;

    Ok(())
}

async fn migrate_oauth_client_foreign_keys(pool: &Pool<Sqlite>) -> Result<(), sqlx::Error> {
    let mut tx = pool.begin().await?;

    sqlx::query("PRAGMA foreign_keys = OFF")
        .execute(&mut *tx)
        .await?;

    rebuild_oauth_child_table_with_client_fk(
        &mut tx,
        "oauth_authorization_codes",
        r#"
        CREATE TABLE oauth_authorization_codes (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            code_hash TEXT NOT NULL UNIQUE,
            client_id TEXT NOT NULL,
            user_id INTEGER NOT NULL,
            redirect_uri TEXT NOT NULL,
            scopes_json TEXT NOT NULL,
            pkce_code_challenge TEXT,
            pkce_method TEXT,
            expires_at DATETIME NOT NULL,
            used_at DATETIME,
            created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
            FOREIGN KEY (client_id) REFERENCES oauth_clients(client_id) ON DELETE CASCADE,
            FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
        );
        CREATE INDEX IF NOT EXISTS idx_oauth_authorization_codes_code_hash
            ON oauth_authorization_codes(code_hash);
        CREATE INDEX IF NOT EXISTS idx_oauth_authorization_codes_client_id
            ON oauth_authorization_codes(client_id);
        CREATE INDEX IF NOT EXISTS idx_oauth_authorization_codes_user_id
            ON oauth_authorization_codes(user_id);
        "#,
        r#"
        INSERT INTO oauth_authorization_codes (
            id, code_hash, client_id, user_id, redirect_uri, scopes_json,
            pkce_code_challenge, pkce_method, expires_at, used_at, created_at
        )
        SELECT
            id, code_hash, client_id, user_id, redirect_uri, scopes_json,
            pkce_code_challenge, pkce_method, expires_at, used_at, created_at
        FROM oauth_authorization_codes_old
        WHERE client_id IN (SELECT client_id FROM oauth_clients)
        "#,
    )
    .await?;

    rebuild_oauth_child_table_with_client_fk(
        &mut tx,
        "oauth_access_tokens",
        r#"
        CREATE TABLE oauth_access_tokens (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            token_hash TEXT NOT NULL UNIQUE,
            client_id TEXT NOT NULL,
            user_id INTEGER NOT NULL,
            scopes_json TEXT NOT NULL,
            expires_at DATETIME NOT NULL,
            revoked_at DATETIME,
            created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
            FOREIGN KEY (client_id) REFERENCES oauth_clients(client_id) ON DELETE CASCADE,
            FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
        );
        CREATE INDEX IF NOT EXISTS idx_oauth_access_tokens_token_hash
            ON oauth_access_tokens(token_hash);
        CREATE INDEX IF NOT EXISTS idx_oauth_access_tokens_client_id
            ON oauth_access_tokens(client_id);
        CREATE INDEX IF NOT EXISTS idx_oauth_access_tokens_user_id
            ON oauth_access_tokens(user_id);
        "#,
        r#"
        INSERT INTO oauth_access_tokens (
            id, token_hash, client_id, user_id, scopes_json, expires_at, revoked_at, created_at
        )
        SELECT
            id, token_hash, client_id, user_id, scopes_json, expires_at, revoked_at, created_at
        FROM oauth_access_tokens_old
        WHERE client_id IN (SELECT client_id FROM oauth_clients)
        "#,
    )
    .await?;

    rebuild_oauth_child_table_with_client_fk(
        &mut tx,
        "oauth_refresh_tokens",
        r#"
        CREATE TABLE oauth_refresh_tokens (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            token_hash TEXT NOT NULL UNIQUE,
            client_id TEXT NOT NULL,
            user_id INTEGER NOT NULL,
            scopes_json TEXT NOT NULL,
            expires_at DATETIME NOT NULL,
            revoked_at DATETIME,
            created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
            FOREIGN KEY (client_id) REFERENCES oauth_clients(client_id) ON DELETE CASCADE,
            FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
        );
        CREATE INDEX IF NOT EXISTS idx_oauth_refresh_tokens_token_hash
            ON oauth_refresh_tokens(token_hash);
        CREATE INDEX IF NOT EXISTS idx_oauth_refresh_tokens_client_id
            ON oauth_refresh_tokens(client_id);
        CREATE INDEX IF NOT EXISTS idx_oauth_refresh_tokens_user_id
            ON oauth_refresh_tokens(user_id);
        "#,
        r#"
        INSERT INTO oauth_refresh_tokens (
            id, token_hash, client_id, user_id, scopes_json, expires_at, revoked_at, created_at
        )
        SELECT
            id, token_hash, client_id, user_id, scopes_json, expires_at, revoked_at, created_at
        FROM oauth_refresh_tokens_old
        WHERE client_id IN (SELECT client_id FROM oauth_clients)
        "#,
    )
    .await?;

    rebuild_oauth_child_table_with_client_fk(
        &mut tx,
        "oauth_consents",
        r#"
        CREATE TABLE oauth_consents (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            user_id INTEGER NOT NULL,
            client_id TEXT NOT NULL,
            scopes_json TEXT NOT NULL,
            granted_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
            revoked_at DATETIME,
            FOREIGN KEY (client_id) REFERENCES oauth_clients(client_id) ON DELETE CASCADE,
            FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
        );
        CREATE INDEX IF NOT EXISTS idx_oauth_consents_user_client
            ON oauth_consents(user_id, client_id);
        "#,
        r#"
        INSERT INTO oauth_consents (
            id, user_id, client_id, scopes_json, granted_at, revoked_at
        )
        SELECT
            id, user_id, client_id, scopes_json, granted_at, revoked_at
        FROM oauth_consents_old
        WHERE client_id IN (SELECT client_id FROM oauth_clients)
        "#,
    )
    .await?;

    sqlx::query("PRAGMA foreign_keys = ON")
        .execute(&mut *tx)
        .await?;

    tx.commit().await?;
    Ok(())
}

async fn migrate_workout_templates(pool: &Pool<Sqlite>) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS workout_templates (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            user_id INTEGER NOT NULL,
            name TEXT NOT NULL,
            created_at DATETIME NOT NULL,
            updated_at DATETIME NOT NULL,
            FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
        );

        CREATE INDEX IF NOT EXISTS idx_workout_templates_user_updated
            ON workout_templates(user_id, updated_at DESC, id DESC);

        CREATE TABLE IF NOT EXISTS workout_template_exercises (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            user_id INTEGER NOT NULL,
            template_id INTEGER NOT NULL,
            position INTEGER NOT NULL,
            exercise_type TEXT NOT NULL,
            notes TEXT,
            per_side_weight INTEGER NOT NULL DEFAULT 0,
            split_weight INTEGER NOT NULL DEFAULT 0,
            FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE,
            FOREIGN KEY (template_id) REFERENCES workout_templates(id) ON DELETE CASCADE
        );

        CREATE INDEX IF NOT EXISTS idx_workout_template_exercises_user_template
            ON workout_template_exercises(user_id, template_id, position, id);

        CREATE TABLE IF NOT EXISTS workout_template_exercise_settings (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            user_id INTEGER NOT NULL,
            template_exercise_id INTEGER NOT NULL,
            setting_key TEXT NOT NULL,
            setting_value TEXT NOT NULL,
            FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE,
            FOREIGN KEY (template_exercise_id) REFERENCES workout_template_exercises(id) ON DELETE CASCADE
        );

        CREATE INDEX IF NOT EXISTS idx_workout_template_settings_user_exercise
            ON workout_template_exercise_settings(user_id, template_exercise_id, id);
        "#,
    )
    .execute(pool)
    .await?;

    Ok(())
}

async fn migrate_mcp_tokens(pool: &Pool<Sqlite>) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS mcp_tokens (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            user_id INTEGER NOT NULL,
            name TEXT NOT NULL,
            token_hash TEXT NOT NULL UNIQUE,
            scopes_json TEXT NOT NULL,
            expires_at DATETIME,
            revoked_at DATETIME,
            last_used_at DATETIME,
            created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
            FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
        );

        CREATE INDEX IF NOT EXISTS idx_mcp_tokens_user_id
            ON mcp_tokens(user_id);
        CREATE INDEX IF NOT EXISTS idx_mcp_tokens_token_hash
            ON mcp_tokens(token_hash);
        "#,
    )
    .execute(pool)
    .await?;

    Ok(())
}

async fn rebuild_oauth_child_table_with_client_fk<'a>(
    tx: &mut sqlx::Transaction<'a, Sqlite>,
    table_name: &str,
    create_sql: &str,
    copy_sql: &str,
) -> Result<(), sqlx::Error> {
    let old_table_name = format!("{table_name}_old");

    sqlx::query(&format!(
        "ALTER TABLE {table_name} RENAME TO {old_table_name}"
    ))
    .execute(&mut **tx)
    .await?;

    sqlx::query(create_sql).execute(&mut **tx).await?;
    sqlx::query(copy_sql).execute(&mut **tx).await?;
    sqlx::query(&format!("DROP TABLE {old_table_name}"))
        .execute(&mut **tx)
        .await?;

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

async fn migrate_multi_user_schema(pool: &Pool<Sqlite>) -> Result<(), sqlx::Error> {
    let app_env = std::env::var("APP_ENV")
        .unwrap_or_else(|_| "development".to_string())
        .to_lowercase();

    let bootstrap_username = std::env::var("BOOTSTRAP_ADMIN_USERNAME")
        .ok()
        .filter(|v| !v.trim().is_empty())
        .unwrap_or_else(|| "admin".to_string());
    let bootstrap_password = std::env::var("BOOTSTRAP_ADMIN_PASSWORD")
        .ok()
        .filter(|v| !v.trim().is_empty());

    let allow_default_bootstrap = matches!(app_env.as_str(), "development" | "test" | "local");
    if !allow_default_bootstrap && bootstrap_password.is_none() {
        return Err(sqlx::Error::Protocol(
            "schema v5 requires BOOTSTRAP_ADMIN_PASSWORD outside development/test/local".into(),
        ));
    }

    let password_plain = bootstrap_password.unwrap_or_else(|| "admin".to_string());
    let password_hash = password::hash_password(&password_plain).map_err(|e| {
        sqlx::Error::Protocol(format!("Failed to hash bootstrap admin password: {e}"))
    })?;

    let mut tx = pool.begin().await?;

    sqlx::query("PRAGMA foreign_keys = OFF")
        .execute(&mut *tx)
        .await?;

    // Auth tables
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS users (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            username TEXT NOT NULL,
            password_hash TEXT NOT NULL,
            role TEXT NOT NULL CHECK(role IN ('admin','user')),
            failed_login_count INTEGER NOT NULL DEFAULT 0,
            locked_until DATETIME,
            disabled_at DATETIME,
            must_change_password INTEGER NOT NULL DEFAULT 0,
            created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
            updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
        );

        CREATE UNIQUE INDEX IF NOT EXISTS idx_users_username_lower ON users(LOWER(username));

        CREATE TRIGGER IF NOT EXISTS trg_users_updated_at
        AFTER UPDATE ON users
        BEGIN
            UPDATE users SET updated_at = CURRENT_TIMESTAMP WHERE id = NEW.id;
        END;

        CREATE TABLE IF NOT EXISTS sessions (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            user_id INTEGER NOT NULL,
            session_hash TEXT NOT NULL UNIQUE,
            created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
            last_seen_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
            expires_at DATETIME NOT NULL,
            revoked_at DATETIME,
            rotated_from_session_id INTEGER,
            user_agent TEXT,
            ip TEXT,
            FOREIGN KEY (user_id) REFERENCES users(id)
        );

        CREATE INDEX IF NOT EXISTS idx_sessions_user_id ON sessions(user_id);
        CREATE INDEX IF NOT EXISTS idx_sessions_expires_at ON sessions(expires_at);
        "#,
    )
    .execute(&mut *tx)
    .await?;

    let user_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM users")
        .fetch_one(&mut *tx)
        .await?;

    if user_count == 0 {
        sqlx::query(
            r#"
            INSERT INTO users (username, password_hash, role, must_change_password)
            VALUES (?, ?, 'admin', 1)
            "#,
        )
        .bind(bootstrap_username)
        .bind(password_hash)
        .execute(&mut *tx)
        .await?;
    }

    let default_user_id: i64 = sqlx::query_scalar("SELECT id FROM users ORDER BY id ASC LIMIT 1")
        .fetch_one(&mut *tx)
        .await?;

    // Rebuild domain tables to add NOT NULL user_id with FKs.
    // Workouts
    sqlx::query("ALTER TABLE workouts RENAME TO workouts_old")
        .execute(&mut *tx)
        .await?;
    sqlx::query(
        r#"
        CREATE TABLE workouts (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            user_id INTEGER NOT NULL,
            date DATETIME NOT NULL,
            start_time DATETIME NOT NULL,
            end_time DATETIME NOT NULL,
            notes TEXT,
            feedback TEXT CHECK(feedback IN ('😊', '😐', '😞') OR feedback IS NULL),
            timezone_offset_minutes INTEGER,
            FOREIGN KEY (user_id) REFERENCES users(id)
        );
        CREATE INDEX IF NOT EXISTS idx_workouts_user_date ON workouts(user_id, date DESC);
        "#,
    )
    .execute(&mut *tx)
    .await?;
    sqlx::query(
        r#"
        INSERT INTO workouts (id, user_id, date, start_time, end_time, notes, feedback, timezone_offset_minutes)
        SELECT id, ?, date, start_time, end_time, notes, feedback, timezone_offset_minutes
        FROM workouts_old
        "#,
    )
    .bind(default_user_id)
    .execute(&mut *tx)
    .await?;

    // Exercises
    sqlx::query("ALTER TABLE exercises RENAME TO exercises_old")
        .execute(&mut *tx)
        .await?;
    sqlx::query(
        r#"
        CREATE TABLE exercises (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            user_id INTEGER NOT NULL,
            workout_id INTEGER NOT NULL,
            exercise_type TEXT NOT NULL,
            start_time DATETIME NOT NULL,
            end_time DATETIME NOT NULL,
            notes TEXT,
            per_side_weight INTEGER NOT NULL DEFAULT 0,
            split_weight INTEGER NOT NULL DEFAULT 0,
            FOREIGN KEY (user_id) REFERENCES users(id),
            FOREIGN KEY (workout_id) REFERENCES workouts(id) ON DELETE CASCADE
        );
        CREATE INDEX IF NOT EXISTS idx_exercises_user_workout_id_composite ON exercises(user_id, workout_id, id);
        "#,
    )
    .execute(&mut *tx)
    .await?;
    sqlx::query(
        r#"
        INSERT INTO exercises (id, user_id, workout_id, exercise_type, start_time, end_time, notes, per_side_weight, split_weight)
        SELECT id, ?, workout_id, exercise_type, start_time, end_time, notes, per_side_weight, split_weight
        FROM exercises_old
        "#,
    )
    .bind(default_user_id)
    .execute(&mut *tx)
    .await?;

    // Sets
    sqlx::query("ALTER TABLE sets RENAME TO sets_old")
        .execute(&mut *tx)
        .await?;
    sqlx::query(
        r#"
        CREATE TABLE sets (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            user_id INTEGER NOT NULL,
            exercise_id INTEGER NOT NULL,
            reps INTEGER NOT NULL,
            weight REAL NOT NULL,
            weight_left REAL,
            weight_right REAL,
            notes TEXT,
            FOREIGN KEY (user_id) REFERENCES users(id),
            FOREIGN KEY (exercise_id) REFERENCES exercises(id) ON DELETE CASCADE
        );
        CREATE INDEX IF NOT EXISTS idx_sets_user_exercise_id_composite ON sets(user_id, exercise_id, id);
        "#,
    )
    .execute(&mut *tx)
    .await?;
    sqlx::query(
        r#"
        INSERT INTO sets (id, user_id, exercise_id, reps, weight, weight_left, weight_right, notes)
        SELECT id, ?, exercise_id, reps, weight, weight_left, weight_right, notes
        FROM sets_old
        "#,
    )
    .bind(default_user_id)
    .execute(&mut *tx)
    .await?;

    // Exercise settings (may not exist on very old DBs, but v2 should have created it).
    let has_settings: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='exercise_settings'",
    )
    .fetch_one(&mut *tx)
    .await?;
    if has_settings > 0 {
        sqlx::query("ALTER TABLE exercise_settings RENAME TO exercise_settings_old")
            .execute(&mut *tx)
            .await?;
    }
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS exercise_settings (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            user_id INTEGER NOT NULL,
            exercise_id INTEGER NOT NULL,
            setting_key TEXT NOT NULL,
            setting_value TEXT NOT NULL,
            FOREIGN KEY (user_id) REFERENCES users(id),
            FOREIGN KEY (exercise_id) REFERENCES exercises(id) ON DELETE CASCADE
        );
        CREATE INDEX IF NOT EXISTS idx_exercise_settings_user_exercise_id_composite
            ON exercise_settings(user_id, exercise_id, id);
        "#,
    )
    .execute(&mut *tx)
    .await?;
    if has_settings > 0 {
        sqlx::query(
            r#"
            INSERT INTO exercise_settings (id, user_id, exercise_id, setting_key, setting_value)
            SELECT id, ?, exercise_id, setting_key, setting_value
            FROM exercise_settings_old
            "#,
        )
        .bind(default_user_id)
        .execute(&mut *tx)
        .await?;
    }

    // Drop old tables
    sqlx::query(
        r#"
        DROP TABLE workouts_old;
        DROP TABLE exercises_old;
        DROP TABLE sets_old;
        "#,
    )
    .execute(&mut *tx)
    .await?;

    if has_settings > 0 {
        sqlx::query("DROP TABLE exercise_settings_old")
            .execute(&mut *tx)
            .await?;
    }

    sqlx::query("PRAGMA foreign_keys = ON")
        .execute(&mut *tx)
        .await?;

    tx.commit().await?;
    Ok(())
}

async fn migrate_user_fk_cascades(pool: &Pool<Sqlite>) -> Result<(), sqlx::Error> {
    let workouts_exist: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='workouts'",
    )
    .fetch_one(pool)
    .await?;
    if workouts_exist > 0 {
        let workout_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM workouts")
            .fetch_one(pool)
            .await?;
        if workout_count > 0 {
            backup::create_backup(backup::BackupType::Auto)
                .await
                .map_err(|e| sqlx::Error::Protocol(format!("Failed to create backup: {e}")))?;
        }
    }

    let mut tx = pool.begin().await?;
    sqlx::query("PRAGMA foreign_keys = OFF")
        .execute(&mut *tx)
        .await?;

    // Sessions (user_id FK cascade).
    sqlx::query("ALTER TABLE sessions RENAME TO sessions_old")
        .execute(&mut *tx)
        .await?;
    sqlx::query(
        r#"
        CREATE TABLE sessions (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            user_id INTEGER NOT NULL,
            session_hash TEXT NOT NULL UNIQUE,
            created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
            last_seen_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
            expires_at DATETIME NOT NULL,
            revoked_at DATETIME,
            rotated_from_session_id INTEGER,
            user_agent TEXT,
            ip TEXT,
            FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
        );
        CREATE INDEX IF NOT EXISTS idx_sessions_user_id ON sessions(user_id);
        CREATE INDEX IF NOT EXISTS idx_sessions_expires_at ON sessions(expires_at);
        "#,
    )
    .execute(&mut *tx)
    .await?;
    sqlx::query(
        r#"
        INSERT INTO sessions (id, user_id, session_hash, created_at, last_seen_at, expires_at, revoked_at, rotated_from_session_id, user_agent, ip)
        SELECT id, user_id, session_hash, created_at, last_seen_at, expires_at, revoked_at, rotated_from_session_id, user_agent, ip
        FROM sessions_old
        "#,
    )
    .execute(&mut *tx)
    .await?;

    // Workouts (user_id FK cascade).
    sqlx::query("ALTER TABLE workouts RENAME TO workouts_old")
        .execute(&mut *tx)
        .await?;
    sqlx::query(
        r#"
        CREATE TABLE workouts (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            user_id INTEGER NOT NULL,
            date DATETIME NOT NULL,
            start_time DATETIME NOT NULL,
            end_time DATETIME NOT NULL,
            notes TEXT,
            feedback TEXT CHECK(feedback IN ('😊', '😐', '😞') OR feedback IS NULL),
            timezone_offset_minutes INTEGER,
            FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
        );
        CREATE INDEX IF NOT EXISTS idx_workouts_user_date ON workouts(user_id, date DESC);
        "#,
    )
    .execute(&mut *tx)
    .await?;
    sqlx::query(
        r#"
        INSERT INTO workouts (id, user_id, date, start_time, end_time, notes, feedback, timezone_offset_minutes)
        SELECT id, user_id, date, start_time, end_time, notes, feedback, timezone_offset_minutes
        FROM workouts_old
        "#,
    )
    .execute(&mut *tx)
    .await?;

    // Exercises (user_id FK cascade).
    sqlx::query("ALTER TABLE exercises RENAME TO exercises_old")
        .execute(&mut *tx)
        .await?;
    sqlx::query(
        r#"
        CREATE TABLE exercises (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            user_id INTEGER NOT NULL,
            workout_id INTEGER NOT NULL,
            exercise_type TEXT NOT NULL,
            start_time DATETIME NOT NULL,
            end_time DATETIME NOT NULL,
            notes TEXT,
            per_side_weight INTEGER NOT NULL DEFAULT 0,
            split_weight INTEGER NOT NULL DEFAULT 0,
            FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE,
            FOREIGN KEY (workout_id) REFERENCES workouts(id) ON DELETE CASCADE
        );
        CREATE INDEX IF NOT EXISTS idx_exercises_user_workout_id_composite ON exercises(user_id, workout_id, id);
        "#,
    )
    .execute(&mut *tx)
    .await?;
    sqlx::query(
        r#"
        INSERT INTO exercises (id, user_id, workout_id, exercise_type, start_time, end_time, notes, per_side_weight, split_weight)
        SELECT id, user_id, workout_id, exercise_type, start_time, end_time, notes, per_side_weight, split_weight
        FROM exercises_old
        "#,
    )
    .execute(&mut *tx)
    .await?;

    // Sets (user_id FK cascade).
    sqlx::query("ALTER TABLE sets RENAME TO sets_old")
        .execute(&mut *tx)
        .await?;
    sqlx::query(
        r#"
        CREATE TABLE sets (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            user_id INTEGER NOT NULL,
            exercise_id INTEGER NOT NULL,
            reps INTEGER NOT NULL,
            weight REAL NOT NULL,
            weight_left REAL,
            weight_right REAL,
            notes TEXT,
            FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE,
            FOREIGN KEY (exercise_id) REFERENCES exercises(id) ON DELETE CASCADE
        );
        CREATE INDEX IF NOT EXISTS idx_sets_user_exercise_id_composite ON sets(user_id, exercise_id, id);
        "#,
    )
    .execute(&mut *tx)
    .await?;
    sqlx::query(
        r#"
        INSERT INTO sets (id, user_id, exercise_id, reps, weight, weight_left, weight_right, notes)
        SELECT id, user_id, exercise_id, reps, weight, weight_left, weight_right, notes
        FROM sets_old
        "#,
    )
    .execute(&mut *tx)
    .await?;

    // Exercise settings (user_id FK cascade).
    let has_settings: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='exercise_settings'",
    )
    .fetch_one(&mut *tx)
    .await?;
    if has_settings > 0 {
        sqlx::query("ALTER TABLE exercise_settings RENAME TO exercise_settings_old")
            .execute(&mut *tx)
            .await?;
    }
    sqlx::query(
        r#"
        CREATE TABLE exercise_settings (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            user_id INTEGER NOT NULL,
            exercise_id INTEGER NOT NULL,
            setting_key TEXT NOT NULL,
            setting_value TEXT NOT NULL,
            FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE,
            FOREIGN KEY (exercise_id) REFERENCES exercises(id) ON DELETE CASCADE
        );
        CREATE INDEX IF NOT EXISTS idx_exercise_settings_user_exercise_id_composite
            ON exercise_settings(user_id, exercise_id, id);
        "#,
    )
    .execute(&mut *tx)
    .await?;
    if has_settings > 0 {
        sqlx::query(
            r#"
            INSERT INTO exercise_settings (id, user_id, exercise_id, setting_key, setting_value)
            SELECT id, user_id, exercise_id, setting_key, setting_value
            FROM exercise_settings_old
            "#,
        )
        .execute(&mut *tx)
        .await?;
    }

    sqlx::query(
        r#"
        DROP TABLE sessions_old;
        DROP TABLE workouts_old;
        DROP TABLE exercises_old;
        DROP TABLE sets_old;
        "#,
    )
    .execute(&mut *tx)
    .await?;
    if has_settings > 0 {
        sqlx::query("DROP TABLE exercise_settings_old")
            .execute(&mut *tx)
            .await?;
    }

    sqlx::query("PRAGMA foreign_keys = ON")
        .execute(&mut *tx)
        .await?;

    tx.commit().await?;
    Ok(())
}
