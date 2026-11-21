use axum::{Json, Router, extract::State, routing::get};
use serde_json::json;
use sqlx::postgres::PgPoolOptions;

mod config;
mod db;
mod service;


#[tokio::main]
async fn main() {
    
    let _ = dotenvy::dotenv();

    let cfg = match config::Config::from_env() {
        Ok(cfg) => cfg,
        Err(msg) => {
            eprintln!("configuration error: {msg}");
            std::process::exit(1);
        }
    };

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .acquire_timeout(std::time::Duration::from_secs(5))
        .connect(&cfg.database_url)
        .await
        .unwrap_or_else(|err| {
            eprintln!("cannot connect to the database: {err}");
            std::process::exit(1);
        });

    let app = Router::new()
        .route("/health", get(health))
        .with_state(pool);

    let addr = std::net::SocketAddr::from(([0, 0, 0, 0], cfg.port));
    let listener = tokio::net::TcpListener::bind(addr).await.expect("bind failed");
    tracing::info!("listening on http://{addr}");
    axum::serve(listener, app).await.expect("server failed");
    
}

async fn health(State(pool): State<sqlx::PgPool>) -> Json<serde_json::Value> {
    match db::db_health(&pool).await {
        Ok(()) => Json(json!({ "status": "UP" })),
        Err(_) => Json(json!({ "status": "DOWN" })),
    }
}

