//! Simplified REST API for L{CORE} Event Indexer

use crate::{error::ApiError, models::*, AppState};
use axum::{
    extract::{State},
    response::Json,
    routing::get,
    Router,
    serve,
};
use serde::{Deserialize, Serialize};
use std::{net::SocketAddr, sync::Arc};
use tower_http::cors::CorsLayer;
use tracing::info;

#[derive(Debug, Deserialize)]
pub struct PaginationQuery {
    #[serde(default = "default_page")]
    pub page: u32,
    #[serde(default = "default_limit")]
    pub limit: u32,
}

fn default_page() -> u32 {
    1
}

fn default_limit() -> u32 {
    20
}

#[derive(Debug, Serialize)]
pub struct PaginatedResponse<T> {
    pub data: Vec<T>,
    pub page: u32,
    pub limit: u32,
    pub total: i64,
}

pub async fn run_server(state: Arc<AppState>) -> Result<(), ApiError> {
    let app = Router::new()
        .route("/health", get(health_check))
        .route("/stats", get(get_stats))
        .layer(CorsLayer::permissive())
        .with_state(state.clone());
    
    let addr = SocketAddr::from(([0, 0, 0, 0], state.config.api_port));
    info!("API server listening on {}", addr);
    
    let listener = tokio::net::TcpListener::bind(&addr).await
        .map_err(|e| ApiError::Internal(e.to_string()))?;
    
    serve(listener, app)
        .await
        .map_err(|e| ApiError::Internal(e.to_string()))?;
    
    Ok(())
}

async fn health_check(State(state): State<Arc<AppState>>) -> Result<Json<HealthResponse>, ApiError> {
    let latest_block = *state.latest_block.read().await;
    
    Ok(Json(HealthResponse {
        status: "healthy".to_string(),
        latest_block,
    }))
}

async fn get_stats(State(state): State<Arc<AppState>>) -> Result<Json<StatsResponse>, ApiError> {
    let latest_block = *state.latest_block.read().await;
    
    Ok(Json(StatsResponse {
        verifier_count: 0,
        device_count: 0,
        data_submission_count: 0,
        latest_block,
    }))
}
