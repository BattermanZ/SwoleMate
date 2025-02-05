use actix_cors::Cors;
use actix_web::{middleware::Logger, App, HttpServer};
use dotenv::dotenv;
use env_logger::Env;
use log::{error, info};
use sqlx::sqlite::SqlitePoolOptions;
use std::env;

mod models;
mod routes;
mod db;
mod errors;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Load environment variables
    dotenv().ok();
    
    // Setup structured logging
    env_logger::init_from_env(Env::default()
        .default_filter_or("info")
        .default_write_style_or("json"));
    
    info!("Starting SwoleMate server...");

    // Get database URL from environment
    let database_url = env::var("DATABASE_URL")
        .unwrap_or_else(|_| "sqlite:database/swolemate.db".to_string());

    // Setup database connection pool
    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await
        .unwrap_or_else(|e| {
            error!("Failed to connect to database: {}", e);
            panic!("Database connection failed");
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

    info!("Server will start on port {}", port);

    // Create and start HTTP server
    HttpServer::new(move || {
        // Configure CORS
        let cors = Cors::default()
            .allow_any_origin()
            .allow_any_method()
            .allow_any_header();

        App::new()
            .wrap(cors)
            .wrap(Logger::new(
                r#"{"time": "%t", "remote_ip": "%a", "request": "%r", "status": "%s", "duration": "%D", "user_agent": "%{User-Agent}i", "request_id": "%{X-Request-ID}i"}"#
            ))
            .app_data(actix_web::web::Data::new(database.clone()))
            .configure(routes::config)
    })
    .bind(("127.0.0.1", port))?
    .run()
    .await
} 