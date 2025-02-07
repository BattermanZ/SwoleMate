use actix_cors::Cors;
use actix_web::{middleware::Logger, App, HttpServer};
use log::{error, info, LevelFilter};
use sqlx::sqlite::SqlitePoolOptions;
use std::env;
use std::fs;
use std::io::Write;
use std::path::Path;
use chrono::Local;
use std::fs::File;

mod models;
mod routes;
mod db;
mod errors;
mod middleware;

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

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Create logs directory if it doesn't exist
    let logs_dir = Path::new("logs");
    if !logs_dir.exists() {
        fs::create_dir(logs_dir)?;
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
    info!("Current working directory: {}", std::env::current_dir()?.display());
    
    match find_env_file() {
        Some(env_path) => {
            info!("Environment loaded successfully from {}", env_path);
            // Verify some key environment variables were loaded
            info!("Verifying environment variables...");
            if let Ok(db_url) = env::var("DATABASE_URL") {
                info!("DATABASE_URL is set to: {}", db_url);
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
    let database_url = env::var("DATABASE_URL")
        .unwrap_or_else(|_| "sqlite:database/swolemate.db".to_string());
    info!("Using database: {}", database_url);

    // Ensure database directory exists and create if needed
    let db_path = Path::new("database");
    if !db_path.exists() {
        fs::create_dir_all(db_path)?;
        info!("Created database directory at: {}", db_path.display());
    }

    // Extract SQLite file path from URL and create if needed
    let db_file = database_url.trim_start_matches("sqlite:");
    let db_file = Path::new(db_file);
    if !db_file.exists() {
        File::create(db_file)?;
        info!("Created new database file at: {}", db_file.display());
    }

    // Setup database connection pool
    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await
        .unwrap_or_else(|e| {
            error!("Failed to connect to database: {}", e);
            panic!("Database connection failed");
        });

    // Enable foreign key support
    sqlx::query!("PRAGMA foreign_keys = ON;")
        .execute(&pool)
        .await
        .unwrap_or_else(|e| {
            error!("Failed to enable foreign keys: {}", e);
            panic!("Failed to enable foreign keys");
        });

    // Run database migrations
    match sqlx::migrate!("./database/migrations").run(&pool).await {
        Ok(_) => info!("Database migrations completed successfully"),
        Err(e) => {
            error!("Failed to run database migrations: {}", e);
            panic!("Migration failed");
        }
    }

    // Create database instance
    let database = db::Database::new(pool.clone());

    // Get server port from environment
    let port = env::var("SERVER_PORT")
        .unwrap_or_else(|_| "2469".to_string())
        .parse::<u16>()
        .expect("SERVER_PORT must be a valid port number");

    // Get frontend URL from environment
    let frontend_url = env::var("FRONTEND_URL")
        .unwrap_or_else(|_| "http://localhost:2470".to_string());
    info!("Allowing CORS for frontend URL: {}", frontend_url);

    info!("Server starting on port {}", port);

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
            .wrap(Logger::new(
                "[%t] %s %r - %D ms - %a - %{User-Agent}i"
            ))
            .wrap(middleware::RequestLogger)
            .app_data(actix_web::web::Data::new(database.clone()))
            .configure(routes::config)
    })
    .bind(("0.0.0.0", port))?
    .run()
    .await
} 