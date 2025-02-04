use actix_cors::Cors;
use actix_web::{middleware::Logger, App, HttpServer};
use env_logger::Env;
use log::info;

mod models;
mod routes;
mod db;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Initialize environment variables
    dotenv::dotenv().ok();
    
    // Setup logging with env_logger
    env_logger::init_from_env(Env::default().default_filter_or("info"));
    info!("Starting SwoleMate server...");

    HttpServer::new(|| {
        // Configure CORS
        let cors = Cors::default()
            .allow_any_origin()
            .allow_any_method()
            .allow_any_header();

        App::new()
            .wrap(cors)
            .wrap(Logger::default())
            .wrap(Logger::new("%a %r %s %b %{Referer}i %{User-Agent}i %T"))
            // Routes will be added here
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
} 