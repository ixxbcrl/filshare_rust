mod db;
mod handlers;
mod models;
mod storage;

use axum::{
    routing::{delete, get, post},
    Router,
};
use std::env;
use std::path::PathBuf;
use tower_http::cors::{Any, CorsLayer};
use tower_http::trace::TraceLayer;
use tracing::info;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() {
    // Initialize tracing
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "fileshare_rust=debug,tower_http=debug,axum=trace".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Load environment variables
    dotenv::dotenv().ok();

    // Configuration
    let database_url = env::var("DATABASE_URL").unwrap_or_else(|_| "sqlite:./files.db".to_string());
    let upload_dir = PathBuf::from(
        env::var("UPLOAD_DIR").unwrap_or_else(|_| "./uploads".to_string())
    );
    let port = env::var("PORT")
        .unwrap_or_else(|_| "3000".to_string())
        .parse::<u16>()
        .expect("PORT must be a valid number");

    info!("Initializing file transfer service...");
    info!("Database: {}", database_url);
    info!("Upload directory: {:?}", upload_dir);
    info!("Port: {}", port);

    // Initialize database
    let pool = db::init_db(&database_url)
        .await
        .expect("Failed to initialize database");

    // Initialize file storage
    let storage = storage::FileStorage::new(upload_dir, pool);
    storage.init().await.expect("Failed to initialize storage");

    // Configure CORS for React frontend
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    // Build router
    let app = Router::new()
        .route("/health", get(handlers::health_check))
        .route("/api/files", get(handlers::list_files))
        .route("/api/files", post(handlers::upload_file))
        .route("/api/files/:id", get(handlers::get_file_info))
        .route("/api/files/:id/download", get(handlers::download_file))
        .route("/api/files/:id", delete(handlers::delete_file))
        .route("/api/directories", post(handlers::create_directory))
        .route("/api/directories/:id", get(handlers::get_directory_info))
        .route("/api/directories/:id", delete(handlers::delete_directory))
        .route("/api/bulk-delete", post(handlers::bulk_delete))
        .layer(cors)
        .layer(TraceLayer::new_for_http())
        .with_state(storage);

    let addr = format!("[::]:{}", port);
    info!("Server starting on {}", addr);

    let listener = tokio::net::TcpListener::bind(&addr)
        .await
        .expect("Failed to bind to address");

    info!("File transfer service is ready!");
    info!("API available at http://{}", addr);

    axum::serve(listener, app)
        .await
        .expect("Server failed to start");
}
