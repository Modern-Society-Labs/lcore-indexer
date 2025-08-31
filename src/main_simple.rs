//! L{CORE} Event Indexer Service - Simplified Version
//! 
//! Basic indexer service with health check API only

use anyhow::{Context, Result};
use clap::Parser;
use sqlx::{postgres::PgPoolOptions, Pool, Postgres};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{error, info};

mod api_simple;
use api_simple as api;
mod config;
mod error;
mod models;

use config::Config;

#[derive(Parser)]
#[command(name = "lcore-indexer")]
#[command(about = "L{CORE} Event Indexer Service", long_about = None)]
struct Args {
    /// Configuration file path
    #[arg(short, long, default_value = "indexer.toml")]
    config: String,
    
    /// Enable verbose logging
    #[arg(short, long)]
    verbose: bool,
}

/// Application state
struct AppState {
    db: Pool<Postgres>,
    config: Config,
    latest_block: Arc<RwLock<u64>>,
}

#[tokio::main]
async fn main() -> Result<()> {
    // Parse command line arguments
    let args = Args::parse();
    
    // Initialize logging
    let filter = if args.verbose { "debug" } else { "info" };
    tracing_subscriber::fmt()
        .with_env_filter(filter)
        .init();
    
    info!("Starting L{{CORE}} Event Indexer");
    
    // Load configuration (try file first, then environment variables)
    let config = Config::load(&args.config)
        .or_else(|_| {
            info!("Config file not found, loading from environment variables");
            Config::from_env()
        })?;
    info!("Configuration loaded successfully");
    
    // Connect to database
    let db = PgPoolOptions::new()
        .max_connections(5)
        .connect(&config.database_url)
        .await
        .context("Failed to connect to database")?;
    
    info!("Connected to database");
    
    // Run migrations
    sqlx::migrate!("./migrations")
        .run(&db)
        .await
        .context("Failed to run database migrations")?;
    
    info!("Database migrations completed");
    
    // Create application state
    let state = Arc::new(AppState {
        db,
        config: config.clone(),
        latest_block: Arc::new(RwLock::new(0)),
    });
    
    // Start API server
    let api_handle = tokio::spawn(api::run_server(state.clone()));
    
    info!("L{{CORE}} Event Indexer started successfully");
    info!("API server running on port {}", config.api_port);
    
    // Wait for API server only
    if let Err(e) = api_handle.await {
        error!("API server failed: {:?}", e);
    }
    
    Ok(())
}
