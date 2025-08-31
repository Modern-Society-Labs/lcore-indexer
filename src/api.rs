//! REST API for querying indexed events

use crate::{error::ApiError, models::*, AppState};
use axum::{
    extract::{Path, Query, State},
    response::Json,
    routing::get,
    Router,
    serve,
};
use serde::{Deserialize, Serialize};
use sqlx::Row;
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
        .route("/verifiers", get(get_verifiers))
        // TODO: Re-enable these endpoints after fixing SQLx macros
        // .route("/verifiers/:address/events", get(get_verifier_events))
        // .route("/devices", get(get_devices))
        // .route("/devices/:id", get(get_device))
        // .route("/devices/:id/events", get(get_device_events))
        // .route("/devices/:id/data", get(get_device_data))
        // .route("/data/recent", get(get_recent_data))
        // .route("/ownership-transfers", get(get_ownership_transfers))
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
    // Simplified stats for now - will be populated as events are indexed
    let latest_block = *state.latest_block.read().await;
    
    Ok(Json(StatsResponse {
        verifier_count: 0,
        device_count: 0,
        data_submission_count: 0,
        latest_block,
    }))
}

async fn get_verifiers(
    State(_state): State<Arc<AppState>>,
    Query(pagination): Query<PaginationQuery>,
) -> Result<Json<PaginatedResponse<VerifierInfo>>, ApiError> {
    // Simplified response for now
    Ok(Json(PaginatedResponse {
        data: vec![],
        page: pagination.page,
        limit: pagination.limit,
        total: 0,
    }))
}

async fn get_verifier_events(
    State(state): State<Arc<AppState>>,
    Path(address): Path<String>,
    Query(pagination): Query<PaginationQuery>,
) -> Result<Json<PaginatedResponse<VerifierEvent>>, ApiError> {
    let offset = ((pagination.page - 1) * pagination.limit) as i64;
    let limit = pagination.limit as i64;
    
    let total = sqlx::query_scalar!(
        "SELECT COUNT(*) FROM verifier_events WHERE verifier_address = $1",
        address
    )
    .fetch_one(&state.db)
    .await?
    .unwrap_or(0);
    
    let events = sqlx::query_as!(
        VerifierEvent,
        r#"
        SELECT 
            id,
            verifier_address,
            event_type as "event_type: _",
            timestamp,
            block_number,
            tx_hash,
            created_at
        FROM verifier_events
        WHERE verifier_address = $1
        ORDER BY timestamp DESC
        LIMIT $2 OFFSET $3
        "#,
        address,
        limit,
        offset
    )
    .fetch_all(&state.db)
    .await?;
    
    Ok(Json(PaginatedResponse {
        data: events,
        page: pagination.page,
        limit: pagination.limit,
        total,
    }))
}

async fn get_devices(
    State(state): State<Arc<AppState>>,
    Query(pagination): Query<PaginationQuery>,
) -> Result<Json<PaginatedResponse<DeviceInfo>>, ApiError> {
    let offset = ((pagination.page - 1) * pagination.limit) as i64;
    let limit = pagination.limit as i64;
    
    let total = sqlx::query_scalar!(
        "SELECT COUNT(DISTINCT device_id) FROM device_events WHERE event_type = 'registered'"
    )
    .fetch_one(&state.db)
    .await?
    .unwrap_or(0);
    
    let devices = sqlx::query_as!(
        DeviceInfo,
        r#"
        SELECT DISTINCT 
            device_id,
            FIRST_VALUE(owner_address) OVER (PARTITION BY device_id ORDER BY timestamp DESC) as owner_address,
            MIN(timestamp) as registered_at,
            MAX(device_type) as device_type,
            MAX(zone) as zone
        FROM device_events
        WHERE event_type = 'registered'
        GROUP BY device_id, owner_address, timestamp
        ORDER BY MIN(timestamp) DESC
        LIMIT $1 OFFSET $2
        "#,
        limit,
        offset
    )
    .fetch_all(&state.db)
    .await?;
    
    Ok(Json(PaginatedResponse {
        data: devices,
        page: pagination.page,
        limit: pagination.limit,
        total,
    }))
}

async fn get_device(
    State(state): State<Arc<AppState>>,
    Path(device_id): Path<String>,
) -> Result<Json<DeviceInfo>, ApiError> {
    let device = sqlx::query_as!(
        DeviceInfo,
        r#"
        SELECT 
            device_id,
            owner_address,
            registered_at,
            device_type,
            zone
        FROM (
            SELECT DISTINCT 
                device_id,
                FIRST_VALUE(owner_address) OVER (PARTITION BY device_id ORDER BY timestamp DESC) as owner_address,
                MIN(timestamp) as registered_at,
                MAX(device_type) as device_type,
                MAX(zone) as zone
            FROM device_events
            WHERE device_id = $1 AND event_type = 'registered'
            GROUP BY device_id, owner_address, timestamp
        ) t
        LIMIT 1
        "#,
        device_id
    )
    .fetch_optional(&state.db)
    .await?
    .ok_or(ApiError::NotFound("Device not found".to_string()))?;
    
    Ok(Json(device))
}

async fn get_device_events(
    State(state): State<Arc<AppState>>,
    Path(device_id): Path<String>,
    Query(pagination): Query<PaginationQuery>,
) -> Result<Json<PaginatedResponse<DeviceEvent>>, ApiError> {
    let offset = ((pagination.page - 1) * pagination.limit) as i64;
    let limit = pagination.limit as i64;
    
    let total = sqlx::query_scalar!(
        "SELECT COUNT(*) FROM device_events WHERE device_id = $1",
        device_id
    )
    .fetch_one(&state.db)
    .await?
    .unwrap_or(0);
    
    let events = sqlx::query_as!(
        DeviceEvent,
        r#"
        SELECT 
            id,
            device_id,
            owner_address,
            event_type as "event_type: _",
            device_type,
            zone,
            timestamp,
            block_number,
            tx_hash,
            created_at
        FROM device_events
        WHERE device_id = $1
        ORDER BY timestamp DESC
        LIMIT $2 OFFSET $3
        "#,
        device_id,
        limit,
        offset
    )
    .fetch_all(&state.db)
    .await?;
    
    Ok(Json(PaginatedResponse {
        data: events,
        page: pagination.page,
        limit: pagination.limit,
        total,
    }))
}

async fn get_device_data(
    State(state): State<Arc<AppState>>,
    Path(device_id): Path<String>,
    Query(pagination): Query<PaginationQuery>,
) -> Result<Json<PaginatedResponse<DataSubmission>>, ApiError> {
    let offset = ((pagination.page - 1) * pagination.limit) as i64;
    let limit = pagination.limit as i64;
    
    // Convert device_id to hash (simplified - in production, use proper hashing)
    let device_id_hash = hex::encode(device_id.as_bytes());
    
    let total = sqlx::query_scalar!(
        "SELECT COUNT(*) FROM data_submissions WHERE device_id_hash = $1",
        device_id_hash
    )
    .fetch_one(&state.db)
    .await?
    .unwrap_or(0);
    
    let submissions = sqlx::query_as!(
        DataSubmission,
        r#"
        SELECT 
            id,
            data_hash,
            device_id_hash,
            device_owner,
            timestamp,
            block_number,
            tx_hash,
            created_at
        FROM data_submissions
        WHERE device_id_hash = $1
        ORDER BY timestamp DESC
        LIMIT $2 OFFSET $3
        "#,
        device_id_hash,
        limit,
        offset
    )
    .fetch_all(&state.db)
    .await?;
    
    Ok(Json(PaginatedResponse {
        data: submissions,
        page: pagination.page,
        limit: pagination.limit,
        total,
    }))
}

async fn get_recent_data(
    State(state): State<Arc<AppState>>,
    Query(pagination): Query<PaginationQuery>,
) -> Result<Json<PaginatedResponse<DataSubmission>>, ApiError> {
    let offset = ((pagination.page - 1) * pagination.limit) as i64;
    let limit = pagination.limit as i64;
    
    let total = sqlx::query_scalar!(
        "SELECT COUNT(*) FROM data_submissions"
    )
    .fetch_one(&state.db)
    .await?
    .unwrap_or(0);
    
    let submissions = sqlx::query_as!(
        DataSubmission,
        r#"
        SELECT 
            id,
            data_hash,
            device_id_hash,
            device_owner,
            timestamp,
            block_number,
            tx_hash,
            created_at
        FROM data_submissions
        ORDER BY timestamp DESC
        LIMIT $1 OFFSET $2
        "#,
        limit,
        offset
    )
    .fetch_all(&state.db)
    .await?;
    
    Ok(Json(PaginatedResponse {
        data: submissions,
        page: pagination.page,
        limit: pagination.limit,
        total,
    }))
}

async fn get_ownership_transfers(
    State(state): State<Arc<AppState>>,
    Query(pagination): Query<PaginationQuery>,
) -> Result<Json<PaginatedResponse<OwnershipTransfer>>, ApiError> {
    let offset = ((pagination.page - 1) * pagination.limit) as i64;
    let limit = pagination.limit as i64;
    
    let total = sqlx::query_scalar!(
        "SELECT COUNT(*) FROM ownership_transfers"
    )
    .fetch_one(&state.db)
    .await?
    .unwrap_or(0);
    
    let transfers = sqlx::query_as!(
        OwnershipTransfer,
        r#"
        SELECT 
            id,
            contract_type,
            previous_owner,
            new_owner,
            block_number,
            tx_hash,
            created_at
        FROM ownership_transfers
        ORDER BY created_at DESC
        LIMIT $1 OFFSET $2
        "#,
        limit,
        offset
    )
    .fetch_all(&state.db)
    .await?;
    
    Ok(Json(PaginatedResponse {
        data: transfers,
        page: pagination.page,
        limit: pagination.limit,
        total,
    }))
}
