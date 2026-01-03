use actix_cors::Cors;
use actix_web::{App, HttpServer};
use chrono::{Datelike, Local, NaiveDate, TimeZone, Weekday};
use log::{error, info, LevelFilter};
use sqlx::sqlite::SqlitePoolOptions;
use std::env;
use std::fs;
use std::fs::File;
use std::io::Write;
use std::path::Path;
use tokio::time::{sleep, Duration};

mod backup;
mod db;
mod errors;
mod middleware;
mod models;
mod routes;

fn find_env_file() -> Option<String> {
    let current_dir = std::env::current_dir().ok()?;
    let env_paths = [
        current_dir.join("server.env"),
        current_dir.join("../server.env"),
        current_dir.join("../../server.env"),
    ];

    for path in env_paths.iter() {
        // Try to actually open and read the file to verify it exists and is readable
        if let Ok(contents) = std::fs::read_to_string(path) {
            info!("Found and verified server.env at: {}", path.display());
            // Manually parse and set environment variables
            for line in contents.lines() {
                if line.starts_with('#') || line.is_empty() {
                    continue;
                }
                if let Some((key, value)) = line.split_once('=') {
                    let key = key.trim();
                    let value = value.trim().trim_matches('"');
                    std::env::set_var(key, value);
                }
            }
            return Some(path.to_string_lossy().into_owned());
        }
    }
    None
}

fn local_datetime_at(date: NaiveDate, preferred_hour: u32) -> chrono::DateTime<Local> {
    // Prefer the requested hour, but handle DST "gaps" and ambiguous times safely.
    for hour in [
        preferred_hour,
        preferred_hour.saturating_add(1),
        preferred_hour.saturating_add(2),
    ] {
        if hour > 23 {
            continue;
        }
        match Local.with_ymd_and_hms(date.year(), date.month(), date.day(), hour, 0, 0) {
            chrono::LocalResult::Single(dt) => return dt,
            chrono::LocalResult::Ambiguous(dt, _) => return dt,
            chrono::LocalResult::None => continue,
        }
    }

    // Last resort: midnight of that date in local time (this should always exist).
    match Local.with_ymd_and_hms(date.year(), date.month(), date.day(), 0, 0, 0) {
        chrono::LocalResult::Single(dt) => dt,
        chrono::LocalResult::Ambiguous(dt, _) => dt,
        chrono::LocalResult::None => Local::now(),
    }
}

fn redact_database_url(database_url: &str) -> String {
    if let Some(path) = database_url.strip_prefix("sqlite:") {
        format!("sqlite:{}", path)
    } else {
        "<redacted>".to_string()
    }
}

async fn schedule_backups() {
    info!("Starting automatic backup scheduler");
    loop {
        let now = Local::now();
        let days_until_monday = (7 + Weekday::Mon.num_days_from_monday() as i64
            - now.weekday().num_days_from_monday() as i64)
            % 7;
        let mut monday_date = now.date_naive() + chrono::Duration::days(days_until_monday);
        let mut next_backup = local_datetime_at(monday_date, 1);

        if next_backup <= now {
            monday_date += chrono::Duration::days(7);
            next_backup = local_datetime_at(monday_date, 1);
        }

        let seconds = (next_backup - now).num_seconds().max(0) as u64;
        sleep(Duration::from_secs(seconds)).await;

        info!("Creating automatic backup");
        match backup::create_backup(backup::BackupType::Auto).await {
            Ok(backup_info) => info!("Automatic backup created: {}", backup_info.filename),
            Err(e) => error!("Failed to create automatic backup: {}", e),
        }
    }
}

const INITIAL_SCHEMA: &str = r#"
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

// Define schema updates for future versions
const SCHEMA_UPDATES: &[(i64, &str)] = &[
    // (2, "ALTER TABLE workouts ADD COLUMN new_column TEXT;"),
    // Add more version updates here as needed
];

const SCHEMA_VERSION_TABLE: &str = r#"
CREATE TABLE IF NOT EXISTS schema_version (
    version INTEGER PRIMARY KEY,
    applied_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
);
"#;

async fn setup_schema(pool: &sqlx::Pool<sqlx::Sqlite>) -> Result<(), sqlx::Error> {
    // Create schema_version table
    sqlx::query(SCHEMA_VERSION_TABLE).execute(pool).await?;

    // Check if this is a pre-v1 database by looking for schema_version
    let has_version_table = sqlx::query_scalar!(
        r#"SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='schema_version'"#
    )
    .fetch_one(pool)
    .await?
        > 0;

    // For pre-v1 databases or new databases, we need to verify the table structure
    let needs_schema_update = if !has_version_table {
        true
    } else {
        // Check if version 1 is recorded
        let version_exists =
            sqlx::query_scalar!(r#"SELECT COUNT(*) FROM schema_version WHERE version = 1"#)
                .fetch_one(pool)
                .await?
                == 0;
        version_exists
    };

    if needs_schema_update {
        // For pre-v1 databases, we need to verify each table's structure
        info!("Checking database structure for potential updates...");

        // Backup existing data if tables exist
        let has_workouts = sqlx::query_scalar!(
            r#"SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='workouts'"#
        )
        .fetch_one(pool)
        .await?
            > 0;

        if has_workouts {
            info!("Found existing workout data, creating backup before schema update");
            backup::create_backup(backup::BackupType::Auto)
                .await
                .map_err(|e| sqlx::Error::Protocol(format!("Failed to create backup: {}", e)))?;
        }

        // Apply initial schema
        info!("Applying initial schema...");
        sqlx::query(INITIAL_SCHEMA).execute(pool).await?;

        // Insert version 1 record
        sqlx::query("INSERT OR IGNORE INTO schema_version (version) VALUES (1)")
            .execute(pool)
            .await?;
        info!("Initial schema (version 1) applied successfully");
    }

    // Apply any pending updates
    for (version, update_sql) in SCHEMA_UPDATES {
        let version_exists = sqlx::query_scalar!(
            r#"SELECT COUNT(*) FROM schema_version WHERE version = ?"#,
            *version
        )
        .fetch_one(pool)
        .await?
            > 0;

        if !version_exists {
            info!("Applying schema update version {}", version);
            sqlx::query(update_sql).execute(pool).await?;

            sqlx::query("INSERT INTO schema_version (version) VALUES (?)")
                .bind(*version)
                .execute(pool)
                .await?;

            info!("Successfully applied schema update version {}", version);
        }
    }

    info!("Database schema is up to date");
    Ok(())
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Create logs directory if it doesn't exist
    let logs_dir = Path::new("logs");
    if !logs_dir.exists() {
        fs::create_dir(logs_dir)?;
        info!("Created logs directory at: {}", logs_dir.display());
    }

    // Create backups directory if it doesn't exist
    let backups_dir = Path::new("backups");
    if !backups_dir.exists() {
        fs::create_dir(backups_dir)?;
        info!("Created backups directory at: {}", backups_dir.display());
    }

    // Create database directory if it doesn't exist
    let db_path = Path::new("database");
    if !db_path.exists() {
        fs::create_dir_all(db_path)?;
        info!("Created database directory at: {}", db_path.display());
    }

    // Setup file logging
    let server_log_file = fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open("logs/server.log")?;

    // Setup structured logging with improved readability
    env_logger::builder()
        .format(move |buf, record| {
            let local_time = Local::now().format("%Y-%m-%d - %H:%M:%S");
            writeln!(
                buf,
                "[{}] {} - {} - {}",
                local_time,
                record.level(),
                record.target(),
                record.args()
            )?;

            // Write to file
            if let Ok(mut file) = server_log_file.try_clone() {
                writeln!(
                    file,
                    "[{}] {} - {} - {}",
                    local_time,
                    record.level(),
                    record.target(),
                    record.args()
                )?;
            }

            Ok(())
        })
        .filter(None, LevelFilter::Info)
        .parse_env("RUST_LOG")
        .init();

    // Load environment variables
    info!("Looking for server.env file...");
    info!(
        "Current working directory: {}",
        std::env::current_dir()?.display()
    );

    match find_env_file() {
        Some(env_path) => {
            info!("Environment loaded successfully from {}", env_path);
            // Verify some key environment variables were loaded
            info!("Verifying environment variables...");
            if let Ok(db_url) = env::var("DATABASE_URL") {
                info!("DATABASE_URL is set to: {}", redact_database_url(&db_url));
            }
            if let Ok(port) = env::var("SERVER_PORT") {
                info!("SERVER_PORT is set to: {}", port);
            }
        }
        None => {
            error!("Could not find server.env in any of the search paths");
            error!("Using default configuration");
        }
    }

    info!("Starting SwoleMate server...");

    // Get database URL from environment
    let database_url =
        env::var("DATABASE_URL").unwrap_or_else(|_| "sqlite:database/swolemate.db".to_string());
    info!("Using database: {}", redact_database_url(&database_url));

    // Ensure database directory exists and create if needed
    let db_file = database_url.trim_start_matches("sqlite:");
    let db_file = Path::new(db_file);
    if !db_file.exists() {
        File::create(db_file)?;
        info!("Created new database file at: {}", db_file.display());
    }

    // Create a temporary connection to check schema
    let temp_pool = SqlitePoolOptions::new()
        .max_connections(1)
        .connect(&database_url)
        .await
        .map_err(|e| std::io::Error::other(format!("{e}")))?;

    // Setup and update schema
    if let Err(e) = setup_schema(&temp_pool).await {
        error!("Failed to setup/update database schema: {}", e);
        return Err(std::io::Error::other("Database schema setup failed"));
    }

    info!("Database schema is up to date");

    // Setup database connection pool
    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await
        .map_err(|e| std::io::Error::other(format!("{e}")))?;

    // Enable foreign key support and optimize performance
    for pragma in [
        "PRAGMA foreign_keys = ON",
        "PRAGMA journal_mode = WAL",
        "PRAGMA synchronous = NORMAL",
        "PRAGMA mmap_size = 30000000000",
        "PRAGMA auto_vacuum = INCREMENTAL",
        "PRAGMA page_size = 4096",
        "PRAGMA cache_size = -8000",
    ] {
        if let Err(e) = sqlx::query(pragma).execute(&pool).await {
            error!("Failed to set pragma {}: {}", pragma, e);
            return Err(std::io::Error::other("Failed to set database pragma"));
        }
    }

    info!("Database configuration completed successfully");

    // Create database instance
    let database = db::Database::new(pool.clone());

    // Get server port from environment
    let port = env::var("SERVER_PORT")
        .unwrap_or_else(|_| "2469".to_string())
        .parse::<u16>()
        .map_err(|_| {
            std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "SERVER_PORT must be a valid port number",
            )
        })?;

    // Get frontend URL from environment
    let frontend_url =
        env::var("FRONTEND_URL").unwrap_or_else(|_| "http://localhost:2470".to_string());
    info!("Allowing CORS for frontend URL: {}", frontend_url);

    info!("Server starting on port {}", port);

    // Start the backup scheduler in a separate task
    tokio::spawn(schedule_backups());

    // Create and start HTTP server
    HttpServer::new(move || {
        // Configure CORS
        let cors = Cors::default()
            .allowed_origin(&frontend_url)
            .allowed_methods(vec!["GET", "POST", "PUT", "DELETE"])
            .allowed_headers(vec![
                actix_web::http::header::AUTHORIZATION,
                actix_web::http::header::ACCEPT,
                actix_web::http::header::CONTENT_TYPE,
            ])
            .max_age(3600);

        App::new()
            .wrap(cors)
            .wrap(middleware::RequestLogger)
            .app_data(actix_web::web::Data::new(database.clone()))
            .configure(routes::config)
    })
    .bind(("0.0.0.0", port))?
    .run()
    .await
}
