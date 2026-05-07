use actix_cors::Cors;
use actix_web::{middleware::DefaultHeaders, web, App, HttpServer};
use chrono::{Datelike, Local, NaiveDate, TimeZone, Weekday};
use log::{error, info, LevelFilter};
use sqlx::{sqlite::SqlitePoolOptions, Pool, Sqlite};
use std::env;
use std::fs;
use std::fs::File;
use std::io::Write;
use std::path::Path;
use tokio::sync::watch;
use tokio::time::{sleep, Duration};

use swolemate_server::{auth, backup, db, mcp, middleware, oauth, routes, schema};

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

fn next_backup_after(now: chrono::DateTime<Local>) -> chrono::DateTime<Local> {
    let days_until_monday = (7 + Weekday::Mon.num_days_from_monday() as i64
        - now.weekday().num_days_from_monday() as i64)
        % 7;
    let mut monday_date = now.date_naive() + chrono::Duration::days(days_until_monday);
    let mut next_backup = local_datetime_at(monday_date, 1);

    if next_backup <= now {
        monday_date += chrono::Duration::days(7);
        next_backup = local_datetime_at(monday_date, 1);
    }

    next_backup
}

fn sleep_duration_until(
    now: chrono::DateTime<Local>,
    next_backup: chrono::DateTime<Local>,
) -> Duration {
    (next_backup - now)
        .to_std()
        .unwrap_or_else(|_| Duration::from_secs(0))
}

async fn checkpoint_and_close_sqlite_pool(pool: Pool<Sqlite>) -> Result<(), sqlx::Error> {
    let checkpoint_result = sqlx::query("PRAGMA wal_checkpoint(TRUNCATE)")
        .execute(&pool)
        .await;
    pool.close().await;
    checkpoint_result.map(|_| ())
}

async fn schedule_backups(mut shutdown: watch::Receiver<bool>) {
    info!("Starting automatic backup scheduler");
    loop {
        if *shutdown.borrow() {
            info!("Backup scheduler stopping (shutdown requested)");
            break;
        }

        let now = Local::now();
        let next_backup = next_backup_after(now);
        let sleep_duration = sleep_duration_until(now, next_backup);
        tokio::select! {
            _ = sleep(sleep_duration) => {},
            _ = shutdown.changed() => {
                info!("Backup scheduler stopping (shutdown requested)");
                break;
            }
        }

        if *shutdown.borrow() {
            info!("Backup scheduler stopping (shutdown requested)");
            break;
        }

        info!("Creating automatic backup");
        match backup::create_backup(backup::BackupType::Auto).await {
            Ok(backup_info) => info!("Automatic backup created: {}", backup_info.filename),
            Err(e) => error!("Failed to create automatic backup: {}", e),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Duration as ChronoDuration;

    #[tokio::test]
    async fn checkpoint_and_close_sqlite_pool_removes_wal_sidecars() {
        let temp_dir = tempfile::tempdir().expect("tempdir");
        let db_path = temp_dir.path().join("swolemate.db");
        let options = sqlx::sqlite::SqliteConnectOptions::new()
            .filename(&db_path)
            .create_if_missing(true);

        let pool = SqlitePoolOptions::new()
            .max_connections(1)
            .connect_with(options)
            .await
            .expect("connect sqlite");

        for pragma in [
            "PRAGMA journal_mode = WAL",
            "PRAGMA wal_autocheckpoint = 0",
            "CREATE TABLE entries (id INTEGER PRIMARY KEY, name TEXT NOT NULL)",
            "INSERT INTO entries (name) VALUES ('shutdown-cleanup')",
        ] {
            sqlx::query(pragma).execute(&pool).await.expect("query");
        }

        let wal_path = db_path.with_extension("db-wal");
        let shm_path = db_path.with_extension("db-shm");

        assert!(wal_path.exists(), "expected WAL file before cleanup");
        assert!(shm_path.exists(), "expected SHM file before cleanup");

        checkpoint_and_close_sqlite_pool(pool)
            .await
            .expect("checkpoint and close");

        assert!(
            !wal_path.exists() || wal_path.metadata().expect("wal metadata").len() == 0,
            "expected WAL file to be removed or truncated after cleanup"
        );
        assert!(
            !shm_path.exists() || shm_path.metadata().expect("shm metadata").len() == 0,
            "expected SHM file to be removed or truncated after cleanup"
        );
    }

    #[test]
    fn backup_sleep_preserves_subsecond_wait_before_target() {
        let now = local_datetime_at(
            NaiveDate::from_ymd_opt(2026, 5, 4).expect("valid test date"),
            0,
        ) + ChronoDuration::minutes(59)
            + ChronoDuration::seconds(59)
            + ChronoDuration::milliseconds(500);

        let next_backup = next_backup_after(now);
        let sleep_duration = sleep_duration_until(now, next_backup);

        assert_eq!(next_backup, local_datetime_at(now.date_naive(), 1));
        assert!(sleep_duration > Duration::from_millis(0));
        assert!(sleep_duration < Duration::from_secs(1));
    }

    #[test]
    fn next_backup_moves_to_following_week_at_target_time() {
        let now = local_datetime_at(
            NaiveDate::from_ymd_opt(2026, 5, 4).expect("valid test date"),
            1,
        );

        let next_backup = next_backup_after(now);

        assert_eq!(
            next_backup.date_naive(),
            now.date_naive() + ChronoDuration::days(7)
        );
    }
}

async fn schedule_auto_close(
    database: db::Database,
    mut shutdown: watch::Receiver<bool>,
    inactivity_minutes: i64,
    poll_seconds: u64,
) {
    if inactivity_minutes <= 0 {
        info!("Auto-close disabled (inactivity_minutes <= 0)");
        return;
    }

    info!(
        "Starting auto-close scheduler inactivity_minutes={} poll_seconds={}",
        inactivity_minutes, poll_seconds
    );

    loop {
        if *shutdown.borrow() {
            info!("Auto-close scheduler stopping (shutdown requested)");
            break;
        }

        tokio::select! {
            _ = sleep(Duration::from_secs(poll_seconds.max(5))) => {},
            _ = shutdown.changed() => {
                info!("Auto-close scheduler stopping (shutdown requested)");
                break;
            }
        }

        if *shutdown.borrow() {
            info!("Auto-close scheduler stopping (shutdown requested)");
            break;
        }

        match database.auto_close_stale_workouts(inactivity_minutes).await {
            Ok(count) => {
                if count > 0 {
                    info!("Auto-closed {} workout(s) due to inactivity", count);
                }
            }
            Err(e) => error!("Failed to auto-close workouts: {}", e),
        }
    }
}

async fn wait_for_shutdown_signal() {
    #[cfg(unix)]
    {
        use tokio::signal::unix::{signal, SignalKind};

        let mut sigterm = signal(SignalKind::terminate()).expect("SIGTERM handler");
        tokio::select! {
            _ = tokio::signal::ctrl_c() => {},
            _ = sigterm.recv() => {},
        }
    }

    #[cfg(not(unix))]
    {
        let _ = tokio::signal::ctrl_c().await;
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let env_path = find_env_file();

    // Setup structured logging (stdout only).
    env_logger::builder()
        .target(env_logger::Target::Stdout)
        .format(|buf, record| {
            let ts = chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true);
            writeln!(
                buf,
                "[{}] {} {} - {}",
                ts,
                record.level(),
                record.target(),
                record.args()
            )
        })
        .filter(None, LevelFilter::Info)
        .parse_env("RUST_LOG")
        .init();

    let (shutdown_tx, shutdown_rx) = watch::channel(false);

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

    // Load environment variables
    info!("Looking for server.env file...");
    info!(
        "Current working directory: {}",
        std::env::current_dir()?.display()
    );

    match env_path {
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

    let app_env = env::var("APP_ENV")
        .unwrap_or_else(|_| "development".to_string())
        .to_lowercase();
    let session_cfg = auth::SessionConfig::for_env(&app_env);
    let oauth_cfg = oauth::OAuthConfig::from_env();
    let enable_hsts = app_env == "production"
        && env::var("ENABLE_HSTS")
            .ok()
            .map(|v| v == "1" || v.eq_ignore_ascii_case("true"))
            .unwrap_or(false);
    let hsts_max_age = env::var("HSTS_MAX_AGE")
        .ok()
        .and_then(|v| v.parse::<u64>().ok())
        .unwrap_or(31536000);
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
    if let Err(e) = schema::setup_schema(&temp_pool).await {
        error!("Failed to setup/update database schema: {}", e);
        temp_pool.close().await;
        return Err(std::io::Error::other("Database schema setup failed"));
    }
    temp_pool.close().await;

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

    let cors_allowed_origins = env::var("CORS_ALLOWED_ORIGINS")
        .ok()
        .map(|raw| {
            raw.split(',')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect::<Vec<_>>()
        })
        .filter(|v| !v.is_empty());

    if let Some(origins) = cors_allowed_origins.as_ref() {
        info!(
            "Allowing CORS origins from CORS_ALLOWED_ORIGINS: {}",
            origins.join(", ")
        );
    } else {
        info!("Allowing CORS origin from FRONTEND_URL: {}", frontend_url);
    }

    let json_body_limit = env::var("JSON_BODY_LIMIT_BYTES")
        .ok()
        .and_then(|v| v.parse::<usize>().ok())
        .unwrap_or(512 * 1024);

    let concurrency = middleware::ApiConcurrency::from_env();

    let auto_close_inactivity_minutes = env::var("AUTO_CLOSE_INACTIVITY_MINUTES")
        .ok()
        .and_then(|v| v.parse::<i64>().ok())
        .unwrap_or(30);
    let auto_close_poll_seconds = env::var("AUTO_CLOSE_POLL_SECONDS")
        .ok()
        .and_then(|v| v.parse::<u64>().ok())
        .unwrap_or(60);

    info!("Server starting on port {}", port);

    // Start the backup scheduler in a separate task.
    tokio::spawn(schedule_backups(shutdown_rx.clone()));
    tokio::spawn(schedule_auto_close(
        database.clone(),
        shutdown_rx.clone(),
        auto_close_inactivity_minutes,
        auto_close_poll_seconds,
    ));

    // Create and start HTTP server.
    let server = HttpServer::new(move || {
        // Configure CORS
        let mut cors = Cors::default()
            .allowed_methods(vec!["GET", "POST", "PUT", "DELETE", "OPTIONS"])
            .supports_credentials()
            .allowed_headers(vec![
                actix_web::http::header::AUTHORIZATION,
                actix_web::http::header::ACCEPT,
                actix_web::http::header::CONTENT_TYPE,
                actix_web::http::header::HeaderName::from_static("mcp-protocol-version"),
            ])
            .max_age(3600);

        if let Some(origins) = cors_allowed_origins.as_ref() {
            for origin in origins {
                cors = cors.allowed_origin(origin);
            }
        } else {
            cors = cors.allowed_origin(&frontend_url);
        }

        let mut headers = DefaultHeaders::new()
            .add(("X-Content-Type-Options", "nosniff"))
            .add(("X-Frame-Options", "DENY"))
            .add(("Referrer-Policy", "no-referrer"))
            .add((
                "Permissions-Policy",
                "camera=(), microphone=(), geolocation=()",
            ));
        if enable_hsts {
            headers = headers.add((
                "Strict-Transport-Security",
                format!("max-age={hsts_max_age}; includeSubDomains"),
            ));
        }

        App::new()
            .wrap(middleware::SessionAuth::new(
                database.clone(),
                session_cfg.clone(),
            ))
            .wrap(headers)
            .wrap(middleware::RequestLogger)
            .wrap(cors)
            .wrap(concurrency.clone())
            .app_data(web::JsonConfig::default().limit(json_body_limit))
            .app_data(actix_web::web::Data::new(database.clone()))
            .app_data(web::Data::new(session_cfg.clone()))
            .app_data(web::Data::new(oauth_cfg.clone()))
            .configure(routes::config)
            .configure(oauth::routes::config)
            .service(
                web::scope("")
                    .wrap(middleware::McpBearerAuth::new(
                        database.clone(),
                        oauth_cfg.protected_resource_endpoint.clone(),
                    ))
                    .configure(mcp::routes::config),
            )
    })
    .bind(("0.0.0.0", port))?
    .client_request_timeout(Duration::from_secs(15))
    .client_disconnect_timeout(Duration::from_secs(5))
    .keep_alive(Duration::from_secs(75))
    .run();

    let handle = server.handle();

    tokio::spawn(async move {
        wait_for_shutdown_signal().await;
        info!("Shutdown requested");
        let _ = shutdown_tx.send(true);

        let stop = handle.stop(true);
        if tokio::time::timeout(Duration::from_secs(15), stop)
            .await
            .is_err()
        {
            error!("Graceful shutdown timed out; forcing stop");
            handle.stop(false).await;
        }
        info!("Shutdown sequence completed");
    });

    let server_result = server.await;
    if let Err(e) = checkpoint_and_close_sqlite_pool(pool).await {
        error!(
            "Failed to checkpoint and close database during shutdown: {}",
            e
        );
    }

    server_result
}
